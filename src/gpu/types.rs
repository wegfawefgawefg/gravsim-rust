use rand::{thread_rng, Rng};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Particle {
    pub pos: [f32; 2],
    pub vel: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimParams {
    pub target_window: [f32; 4],
    pub sim: [f32; 4],
}

pub fn make_particles(count: u32, width: u32, height: u32) -> Vec<Particle> {
    let spawn_center = [width as f32 * 0.5, height as f32 * 0.5];
    let spawn_radius = [width as f32 * 0.25, height as f32 * 0.25];
    let mut rng = thread_rng();

    let mut particles = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let pos = [
            rng.gen_range((spawn_center[0] - spawn_radius[0])..(spawn_center[0] + spawn_radius[0])),
            rng.gen_range((spawn_center[1] - spawn_radius[1])..(spawn_center[1] + spawn_radius[1])),
        ];
        let vel = [rng.gen_range(-2.0..2.0), rng.gen_range(-2.0..2.0)];
        particles.push(Particle { pos, vel });
    }
    particles
}
