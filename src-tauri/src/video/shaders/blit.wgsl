// Kadro tekstūros piešimas per centruotą quad'ą (CLAUDE.md §4, P2.4–P2.5).
//
// P2.5: `uniforms.scale` nustato quad'o pusę kampų atstumu nuo centro (NDC), kad išlaikytų
// teisingą aspect ratio / integer scaling. Sritis už quad'o ribų lieka nenupiešta — rodo
// render pass'o Clear(BLACK) spalvą (letterbox/pillarbox juodi kraštai).
//
// PASTABA: čia SĄMONINGAI naudojamas TIKRAS quad'as (4 kampai, 2 trikampiai iš 6 viršūnių),
// o NE įprastas „vieno pilno ekrano trikampio" triukas. Priežastis — tas triukas piešia
// perteklinį trikampį, kurio kraštinės iškarpomos TIK prie fiksuotos NDC [-1,1] ribos; kai
// pozicija po to dauginama iš `scale < 1`, iškarpymo riba NEPRISITAIKO (viena kraštinė lieka
// pritvirtinta prie senos NDC ribos, kita susitraukia) — rezultatas asimetriškas quad'as
// (patikrinta vizualiai P2.5 metu: juoda juosta atsirado tik vienoje pusėje). Tikras quad'as
// su fiksuotais 4 kampais šitos klaidos neturi — kampai visada `(±scale.x, ±scale.y)`.

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Uniforms {
    scale: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(2) var<uniform> uniforms: Uniforms;

const CORNER_POSITIONS = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
);
const CORNER_UVS = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
);
const QUAD_INDICES = array<u32, 6>(0u, 1u, 2u, 2u, 1u, 3u);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let corner = QUAD_INDICES[vertex_index];
    let pos = CORNER_POSITIONS[corner];
    out.uv = CORNER_UVS[corner];
    out.clip_position = vec4<f32>(pos.x * uniforms.scale.x, pos.y * uniforms.scale.y, 0.0, 1.0);
    return out;
}

@group(0) @binding(0) var frame_texture: texture_2d<f32>;
@group(0) @binding(1) var frame_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(frame_texture, frame_sampler, in.uv);
}
