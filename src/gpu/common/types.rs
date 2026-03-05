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

pub fn make_chain_particles(
    count: u32,
    width: u32,
    height: u32,
    use_grid_spawn: bool,
) -> Vec<Particle> {
    if use_grid_spawn {
        return make_chain_particles_grid(count, width, height);
    }
    make_chain_particles_random(count, width, height)
}

fn make_chain_particles_random(count: u32, width: u32, height: u32) -> Vec<Particle> {
    let mut rng = thread_rng();
    let mut particles = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let pos = [
            rng.gen_range(0.0..width as f32),
            rng.gen_range(0.0..height as f32),
        ];
        particles.push(Particle {
            pos,
            vel: [0.0, 0.0],
        });
    }

    particles
}

fn make_chain_particles_grid(count: u32, width: u32, height: u32) -> Vec<Particle> {
    if count == 0 {
        return Vec::new();
    }

    let width_f = width as f32;
    let height_f = height as f32;
    let aspect = width_f / height_f.max(1.0);
    let count_usize = count as usize;
    let cols = ((count as f32 * aspect).sqrt().ceil() as usize).max(1);
    let rows = count_usize.div_ceil(cols).max(1);
    let cell_w = width_f / cols as f32;
    let cell_h = height_f / rows as f32;

    let mut particles = Vec::with_capacity(count_usize);
    for i in 0..count_usize {
        let x_idx = i % cols;
        let y_idx = i / cols;
        let x = ((x_idx as f32 + 0.5) * cell_w).clamp(0.0, width_f);
        let y = ((y_idx as f32 + 0.5) * cell_h).clamp(0.0, height_f);
        particles.push(Particle {
            pos: [x, y],
            vel: [0.0, 0.0],
        });
    }

    particles
}
