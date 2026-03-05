struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
};

struct Camera {
    center_zoom: vec4<f32>,
    viewport_grid: vec4<f32>,
};

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> camera: Camera;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    let p = particles[idx].pos;
    let center = camera.center_zoom.xy;
    let zoom = camera.center_zoom.z;
    let viewport = camera.viewport_grid.xy;

    let rel = p - center;
    let x = rel.x * (2.0 * zoom / viewport.x);
    let y = -rel.y * (2.0 * zoom / viewport.y);

    var out: VertexOut;
    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.color = vec4<f32>(1.0, 1.0, 1.0, 0.25);
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}
