struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
};

struct Params {
    target_window: vec4<f32>,
    sim: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> src_particles: array<Particle>;

@group(0) @binding(1)
var<storage, read_write> dst_particles: array<Particle>;

@group(0) @binding(2)
var<uniform> params: Params;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let count = u32(params.sim.z);
    if (i >= count || count == 0u) {
        return;
    }

    var p = src_particles[i];
    let next_i = (i + 1u) % count;
    let target_pos = src_particles[next_i].pos;

    let delta = target_pos - p.pos;
    let dist_sq = dot(delta, delta);
    if (dist_sq > 1e-12) {
        let inv_len = inverseSqrt(dist_sq);
        p.vel = delta * inv_len * params.sim.x;
    } else {
        p.vel = vec2<f32>(0.0, 0.0);
    }
    p.pos += p.vel;

    if (params.sim.y > 0.5) {
        p.pos.x = clamp(p.pos.x, 0.0, params.target_window.z);
        p.pos.y = clamp(p.pos.y, 0.0, params.target_window.w);
    }

    dst_particles[i] = p;
}
