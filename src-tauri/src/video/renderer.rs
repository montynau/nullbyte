//! Emuliatoriaus langas, wgpu Surface ir blit pipeline (CLAUDE.md §10, P2.3–P2.4).
//!
//! Emuliatoriaus vaizdas piešiamas atskirame Tauri `Window` BE webview
//! (`tauri::window::WindowBuilder`, ne `WebviewWindowBuilder`) — leidžia tiesiogiai valdyti
//! wgpu Surface be permatomumo/click-through problemų, kylančių bandant piešti „po" webview
//! (CLAUDE.md §10 „wgpu / Tauri" spąstai).
//!
//! **Surface kūrimas, konfigūravimas ir `present()` — TIK main/UI gijoje.** macOS/Metal
//! reikalauja, kad `NSView` (taigi ir su juo susietas Surface) būtų pasiekiamas tik main
//! gijoje; wgpu tai atspindi panikuodamas, jei pažeidžiama. `Renderer::new` naudoja
//! `tauri::async_runtime::block_on`, kad sinchroniai palauktų `request_adapter`/
//! `request_device` — natūviose (ne-web) platformose šie iš tiesų užbaigiami iškart,
//! `Future` apvalkalas daugiausia web suderinamumui.
//!
//! Blit pipeline (P2.4): kadras iš [`super::frame_buffer`] įkeliamas į `Rgba8Unorm` tekstūrą
//! (`queue.write_texture`), tada nupiešiamas per centruotą quad'ą (be vertex buffer'io, 4
//! kampai iš `vertex_index`) ir sample'inamas fragment shader'yje — žr. `shaders/blit.wgsl`.
//! P2.5: quad'o dydis (aspect ratio / integer scaling) valdomas `scale` uniform'u.

// device()/queue() accessor'ius naudoja tik testai/ateities kodas.
#![allow(dead_code)]

use tauri::window::Window;
use tauri::Runtime;

use super::frame_buffer::VideoFrameData;
use crate::error::AppError;

/// Tekstūros filtravimo režimas. `Nearest` — numatytasis (pixel-perfect, CLAUDE.md §7.5),
/// `Linear` — pasirenkamas nustatymuose (post-MVP UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    #[default]
    Nearest,
    Linear,
}

/// Kadro dydžio skaičiavimo režimas lango viduje (P2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    /// Didžiausias dydis, kuris tilpsta lange, IŠLAIKANT teisingą aspect ratio
    /// (`av_info.geometry.aspect_ratio`, arba `width/height`, jei core'as jo nenurodo).
    /// Nepilnai užpildytos sritys — letterbox/pillarbox juodi kraštai.
    #[default]
    Fit,
    /// Didžiausias SVEIKASIS (1x, 2x, 3x, ...) šaltinio pikselių daugiklis, kuris tilpsta
    /// lange — kiekvienas core'o pikselis atvaizduojamas kaip tolygus NxN blokas, be
    /// interpoliacijos artefaktų (nepriklauso nuo `aspect_ratio` — sąmoningai laiko
    /// pikselius kvadratiniais, kaip įprasta emuliatorių „integer scale" režimuose).
    Integer,
}

/// wgpu būvis, susietas su vienu emuliatoriaus langu.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    nearest_sampler: wgpu::Sampler,
    linear_sampler: wgpu::Sampler,
    filter: FilterMode,

    /// `scale` uniform buferis (P2.5) — vertex shader'yje sutraukia NDC poziciją, kad
    /// išlaikytų aspect ratio / integer scaling. Turinys perrašomas kas kadrą `render()`
    /// pradžioje — pigu (16 baitų), o vengia atskirų „dirty" žymių kiekvienam mutatoriui
    /// (`resize`, `upload_frame`, `set_scale_mode`).
    scale_uniform: wgpu::Buffer,
    scale_mode: ScaleMode,
    /// Paskutinio įkelto kadro `av_info.geometry.aspect_ratio` — `<= 0.0` reiškia
    /// „nenurodyta", naudok `frame_size` santykį.
    aspect_ratio: f32,

    frame_texture: Option<wgpu::Texture>,
    frame_size: (u32, u32),
    bind_group: Option<wgpu::BindGroup>,
}

impl Renderer {
    /// Sukuria wgpu `Instance`/`Surface`/`Adapter`/`Device` duotam langui, sukonfigūruoja
    /// `Surface` jo dabartiniam dydžiui ir paruošia blit pipeline (shader'iai, samplerai).
    ///
    /// # Panic
    /// wgpu panikuoja, jei kviečiama ne pagrindinėje gijoje macOS/Metal atveju — šis metodas
    /// PRIVALO būti kviečiamas iš UI gijos (žr. modulio doc).
    pub fn new<R: Runtime>(window: Window<R>) -> Result<Self, AppError> {
        let size = window
            .inner_size()
            .map_err(|e| AppError::Other(format!("nepavyko gauti lango dydžio: {e}")))?;

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // `Window<R>` implementuoja HasWindowHandle+HasDisplayHandle, tad create_surface()
        // priima ją tiesiogiai — jokio rankinio unsafe raw handle darbo mūsų pusėje.
        let surface = instance
            .create_surface(window)
            .map_err(|e| AppError::Other(format!("nepavyko sukurti wgpu Surface: {e}")))?;

        let adapter = tauri::async_runtime::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ))
        .map_err(|e| AppError::Other(format!("nepavyko rasti tinkamo GPU adapterio: {e}")))?;

        tracing::info!(
            adapter = adapter.get_info().name,
            backend = ?adapter.get_info().backend,
            "wgpu adapteris pasirinktas"
        );

        let (device, queue) =
            tauri::async_runtime::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("nullbyte-device"),
                ..Default::default()
            }))
            .map_err(|e| AppError::Other(format!("nepavyko gauti wgpu Device: {e}")))?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        tracing::info!(
            width = config.width,
            height = config.height,
            ?format,
            "wgpu Surface sukonfigūruotas"
        );

        let (pipeline, bind_group_layout) = create_blit_pipeline(&device, format);
        let nearest_sampler = create_sampler(&device, wgpu::FilterMode::Nearest);
        let linear_sampler = create_sampler(&device, wgpu::FilterMode::Linear);

        // 16 baitų = vec2<f32> scale + vec2<f32> padding (WGSL uniform bloko išlygiavimas).
        let scale_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nullbyte-scale-uniform"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group_layout,
            nearest_sampler,
            linear_sampler,
            filter: FilterMode::default(),
            scale_uniform,
            scale_mode: ScaleMode::default(),
            aspect_ratio: 0.0,
            frame_texture: None,
            frame_size: (0, 0),
            bind_group: None,
        })
    }

    /// Rekonfigūruoja `Surface` naujam lango dydžiui. Ignoruoja `0×0` (minimizuotas langas) —
    /// wgpu panikuotų, jei bandytum konfigūruoti su nuliniu matmeniu.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if self.config.width == width && self.config.height == height {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        tracing::debug!(width, height, "wgpu Surface rekonfigūruotas (resize)");
    }

    /// Nustato tekstūros filtravimo režimą (post-MVP nustatymų ekranui). Įsigalioja nuo
    /// kito [`Renderer::upload_frame`] kvietimo (bind group'as perkuriamas).
    pub fn set_filter(&mut self, filter: FilterMode) {
        if self.filter == filter {
            return;
        }
        self.filter = filter;
        self.bind_group = None; // priverčia ensure_texture perkurti su nauju sampler'iu
    }

    /// Nustato kadro dydžio skaičiavimo režimą (post-MVP nustatymų ekranui, P2.5). Įsigalioja
    /// nuo kito [`Renderer::render`] kvietimo — uniform buferis perrašomas kas kadrą, bind
    /// group'o perkurti nereikia.
    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        self.scale_mode = mode;
    }

    /// Įkelia naują kadrą į GPU tekstūrą (`queue.write_texture`). Tekstūra perkuriama tik
    /// kai pasikeičia dydis (P2.1 nulinio-alokavimo principo tąsa GPU pusėje).
    pub fn upload_frame(&mut self, frame: &VideoFrameData) {
        if frame.width == 0 || frame.height == 0 {
            return;
        }
        self.aspect_ratio = frame.aspect_ratio;
        self.ensure_texture(frame.width, frame.height);
        let Some(texture) = &self.frame_texture else {
            return;
        };

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &frame.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(frame.width * 4),
                rows_per_image: Some(frame.height),
            },
            wgpu::Extent3d {
                width: frame.width,
                height: frame.height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn ensure_texture(&mut self, width: u32, height: u32) {
        if self.frame_size == (width, height)
            && self.frame_texture.is_some()
            && self.bind_group.is_some()
        {
            return;
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nullbyte-frame-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = match self.filter {
            FilterMode::Nearest => &self.nearest_sampler,
            FilterMode::Linear => &self.linear_sampler,
        };

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nullbyte-blit-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.scale_uniform.as_entire_binding(),
                },
            ],
        });

        self.frame_texture = Some(texture);
        self.frame_size = (width, height);
        self.bind_group = Some(bind_group);
    }

    /// Apskaičiuoja `[scale_x, scale_y]` NDC daugiklį (P2.5) — sutraukia pilno ekrano
    /// trikampį taip, kad nupieštas kadras išlaikytų teisingą aspect ratio (`Fit`) arba
    /// būtų sveikasis šaltinio pikselių daugiklis (`Integer`). Grąžina `[1.0, 1.0]`
    /// (pilnas langas), jei dar nėra kadro arba lango dydis nulinis.
    fn compute_scale(&self) -> [f32; 2] {
        let (frame_width, frame_height) = self.frame_size;
        if frame_width == 0
            || frame_height == 0
            || self.config.width == 0
            || self.config.height == 0
        {
            return [1.0, 1.0];
        }

        let window_width = self.config.width as f32;
        let window_height = self.config.height as f32;

        match self.scale_mode {
            ScaleMode::Fit => {
                let target_aspect = if self.aspect_ratio > 0.0 {
                    self.aspect_ratio
                } else {
                    frame_width as f32 / frame_height as f32
                };
                let window_aspect = window_width / window_height;
                if window_aspect > target_aspect {
                    // Langas platesnis už turinį — pillarbox (juosta kairėje/dešinėje).
                    [target_aspect / window_aspect, 1.0]
                } else {
                    // Langas aukštesnis/siauresnis už turinį — letterbox (juosta viršuje/apačioje).
                    [1.0, window_aspect / target_aspect]
                }
            }
            ScaleMode::Integer => {
                let max_scale_x = window_width / frame_width as f32;
                let max_scale_y = window_height / frame_height as f32;
                let integer_scale = max_scale_x.min(max_scale_y).floor().max(1.0);
                let rendered_width = frame_width as f32 * integer_scale;
                let rendered_height = frame_height as f32 * integer_scale;
                [
                    rendered_width / window_width,
                    rendered_height / window_height,
                ]
            }
        }
    }

    /// Nupiešia dabartinę tekstūrą (paskutinį `upload_frame` kadrą) į `Surface` ir
    /// pateikia (`present`). Jei dar nebuvo nė vieno `upload_frame` — nupiešia tuščią
    /// (juodą) langą.
    ///
    /// `PresentMode::AutoVsync` (nustatyta `new()`) užtikrina, kad `present()` sinchronizuotųsi
    /// su ekrano atnaujinimu — be tearing'o (P2.4 acceptance).
    pub fn render(&mut self) -> Result<(), AppError> {
        let scale = self.compute_scale();
        let mut uniform_bytes = [0u8; 16];
        uniform_bytes[0..4].copy_from_slice(&scale[0].to_le_bytes());
        uniform_bytes[4..8].copy_from_slice(&scale[1].to_le_bytes());
        self.queue
            .write_buffer(&self.scale_uniform, 0, &uniform_bytes);

        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(e) => {
                return Err(AppError::Other(format!(
                    "nepavyko gauti Surface texture: {e}"
                )))
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("nullbyte-blit-encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nullbyte-blit-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(bind_group) = &self.bind_group {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                // 6 viršūnės = quad (2 trikampiai), ne 3 — žr. blit.wgsl komentarą, kodėl
                // „pilno ekrano trikampio" triukas netinka scale'inamam (P2.5) quad'ui.
                pass.draw(0..6, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// GPU device — naudos garso/kitų posistemių integracijos, jei kada reikės bendro Device.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// GPU komandų eilė.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Dabartinė Surface konfigūracija (formatas, dydis).
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}

fn create_sampler(device: &wgpu::Device, filter: wgpu::FilterMode) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some(match filter {
            wgpu::FilterMode::Nearest => "nullbyte-nearest-sampler",
            wgpu::FilterMode::Linear => "nullbyte-linear-sampler",
        }),
        mag_filter: filter,
        min_filter: filter,
        ..Default::default()
    })
}

fn create_blit_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/blit.wgsl"));

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("nullbyte-blit-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("nullbyte-blit-pipeline-layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("nullbyte-blit-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    (pipeline, bind_group_layout)
}
