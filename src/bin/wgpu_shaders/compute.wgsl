struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
};

struct Params {
    mouse_window: vec4<f32>,
    sim: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> params: Params;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = u32(params.sim.w);
    if (i >= count) {
        return;
    }

    var p = particles[i];
    let delta = params.mouse_window.xy - p.pos;
    let dist_sq = dot(delta, delta);
    if (dist_sq > 4.0) {
        p.vel += delta * (params.sim.x / dist_sq) * params.sim.y;
    }

    p.pos += p.vel * params.sim.y;

    if (params.sim.z > 0.5) {
        p.pos.x = clamp(p.pos.x, 0.0, params.mouse_window.z);
        p.pos.y = clamp(p.pos.y, 0.0, params.mouse_window.w);
    }

    particles[i] = p;
}
