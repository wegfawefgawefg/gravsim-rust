struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
};

struct Params {
    target_window: vec4<f32>,
    sim: vec4<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    let p = particles[idx].pos;
    let x = (p.x / params.target_window.z) * 2.0 - 1.0;
    let y = 1.0 - (p.y / params.target_window.w) * 2.0;

    var out: VertexOut;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.color = vec4<f32>(1.0, 1.0, 1.0, 0.25);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
