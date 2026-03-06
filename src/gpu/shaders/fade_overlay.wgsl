struct FadeParams {
    rgba: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> fade: FadeParams;

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0),
    );

    var out: VertexOut;
    out.pos = vec4<f32>(pos[idx], 0.0, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return fade.rgba;
}
