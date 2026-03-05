struct Camera {
    center_zoom: vec4<f32>,
    viewport_grid: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOut {
    @builtin(position) pos: vec4<f32>,
};

fn line_alpha(world_coord: f32, spacing: f32, zoom: f32, line_width_px: f32) -> f32 {
    let cell = world_coord / spacing;
    let dist_world = abs(cell - round(cell)) * spacing;
    let dist_px = dist_world * zoom;
    return clamp((line_width_px - dist_px) / line_width_px, 0.0, 1.0);
}

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
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    let center = camera.center_zoom.xy;
    let zoom = camera.center_zoom.z;
    let viewport = camera.viewport_grid.xy;
    let major = camera.viewport_grid.z;
    let minor = camera.viewport_grid.w;

    let world = vec2<f32>(
        center.x + (frag_pos.x - viewport.x * 0.5) / zoom,
        center.y + (frag_pos.y - viewport.y * 0.5) / zoom,
    );

    let minor_lines = max(
        line_alpha(world.x, minor, zoom, 0.75),
        line_alpha(world.y, minor, zoom, 0.75),
    );
    let major_lines = max(
        line_alpha(world.x, major, zoom, 1.3),
        line_alpha(world.y, major, zoom, 1.3),
    );
    let axis_lines = max(
        line_alpha(world.x, major, zoom, 1.8) * select(0.0, 1.0, abs(world.x) < major * 0.5),
        line_alpha(world.y, major, zoom, 1.8) * select(0.0, 1.0, abs(world.y) < major * 0.5),
    );

    let color =
        vec3<f32>(0.02, 0.06, 0.09) * minor_lines +
        vec3<f32>(0.06, 0.16, 0.24) * major_lines +
        vec3<f32>(0.25, 0.35, 0.10) * axis_lines;

    return vec4<f32>(color, 1.0);
}
