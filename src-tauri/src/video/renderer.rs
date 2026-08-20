//! Emuliatoriaus langas ir wgpu Surface (CLAUDE.md §10, P2.3).
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

// device()/queue() accessor'ius pilnai naudos P2.4 blit pipeline.
#![allow(dead_code)]

use tauri::window::Window;
use tauri::Runtime;

use crate::error::AppError;

/// wgpu būvis, susietas su vienu emuliatoriaus langu.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Renderer {
    /// Sukuria wgpu `Instance`/`Surface`/`Adapter`/`Device` duotam langui ir sukonfigūruoja
    /// `Surface` jo dabartiniam dydžiui.
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

        Ok(Self {
            surface,
            device,
            queue,
            config,
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

    /// GPU device — naudos P2.4 blit pipeline.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// GPU komandų eilė — naudos P2.4 blit pipeline.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Dabartinė Surface konfigūracija (formatas, dydis) — naudos P2.4 blit pipeline.
    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}
