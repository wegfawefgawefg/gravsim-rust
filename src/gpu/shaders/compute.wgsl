struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
};

struct Params {
    target_window: vec4<f32>,
    sim: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@group(0) @binding(1)
var<uniform> params: Params;

const MAX_DISPATCH_GROUPS_X: u32 = 65535u;
const WORKGROUP_SIZE_X: u32 = 256u;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x + gid.y * (MAX_DISPATCH_GROUPS_X * WORKGROUP_SIZE_X);
    let count = arrayLength(&particles);
    if (i >= count || count == 0u) {
        return;
    }

    var p = particles[i];
    let delta = params.target_window.xy - p.pos;
    let dist_sq = dot(delta, delta);
    if (dist_sq > 4.0) {
        p.vel += delta * (params.sim.x / dist_sq);
    }

    p.pos += p.vel;

    if (params.sim.y > 0.5) {
        p.pos.x = clamp(p.pos.x, 0.0, params.target_window.z);
        p.pos.y = clamp(p.pos.y, 0.0, params.target_window.w);
    }

    particles[i] = p;
}
