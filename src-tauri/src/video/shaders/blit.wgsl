// Pilno ekrano trikampis + tekstūros sample'inimas (CLAUDE.md §4, P2.4).
//
// Standartinis "full-screen triangle" triukas: 3 viršūnės generuojamos vien iš
// vertex_index (be vertex buffer'io), apimančios visą clip space [-1,1]x[-1,1] —
// du kraštai persidengia už ekrano ribų, bet tai pigiau ir paprasčiau nei quad + indeksai.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    out.uv = vec2<f32>(x, y);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var frame_texture: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(frame_texture, frame_sampler, in.uv);
}
