use glam::Vec2;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};

use crate::config::{CHAIN_USE_GRID_SPAWN, NUM_BODIES, WINDOW_CENTER, WINDOW_DIMS};

pub struct Bodies {
    pub pos: Vec<Vec2>,
    pub vel: Vec<Vec2>,
}

pub fn make_bodies() -> Bodies {
    const INITIAL_VEL_MAX: f32 = 2.0;
    let spawn_start_p = WINDOW_CENTER - WINDOW_CENTER / 2.0;
    let spawn_end_p = WINDOW_CENTER + WINDOW_CENTER / 8.0;
    let mut rng = thread_rng();
    let mut pos = Vec::with_capacity(NUM_BODIES);
    let mut vel = Vec::with_capacity(NUM_BODIES);

    for _ in 0..NUM_BODIES {
        pos.push(Vec2::new(
            rng.gen_range(spawn_start_p.x..spawn_end_p.x),
            rng.gen_range(spawn_start_p.y..spawn_end_p.y),
        ));
        vel.push(Vec2::new(
            rng.gen_range(-INITIAL_VEL_MAX..INITIAL_VEL_MAX),
            rng.gen_range(-INITIAL_VEL_MAX..INITIAL_VEL_MAX),
        ));
    }

    Bodies { pos, vel }
}

pub fn make_chain_bodies(spawn_seed: Option<u64>) -> Bodies {
    let pos = if CHAIN_USE_GRID_SPAWN {
        make_grid_positions(NUM_BODIES)
    } else {
        make_random_positions(NUM_BODIES, spawn_seed)
    };
    let vel = vec![Vec2::ZERO; NUM_BODIES];
    Bodies { pos, vel }
}

fn make_random_positions(count: usize, spawn_seed: Option<u64>) -> Vec<Vec2> {
    let mut pos = Vec::with_capacity(count);

    if let Some(seed) = spawn_seed {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..count {
            pos.push(Vec2::new(
                rng.gen_range(0.0..WINDOW_DIMS.x as f32),
                rng.gen_range(0.0..WINDOW_DIMS.y as f32),
            ));
        }
    } else {
        let mut rng = thread_rng();
        for _ in 0..count {
            pos.push(Vec2::new(
                rng.gen_range(0.0..WINDOW_DIMS.x as f32),
                rng.gen_range(0.0..WINDOW_DIMS.y as f32),
            ));
        }
    }

    pos
}

fn make_grid_positions(count: usize) -> Vec<Vec2> {
    if count == 0 {
        return Vec::new();
    }

    let width = WINDOW_DIMS.x as f32;
    let height = WINDOW_DIMS.y as f32;
    let aspect = width / height.max(1.0);
    let cols = ((count as f32 * aspect).sqrt().ceil() as usize).max(1);
    let rows = count.div_ceil(cols).max(1);
    let cell_w = width / cols as f32;
    let cell_h = height / rows as f32;

    let mut pos = Vec::with_capacity(count);
    for i in 0..count {
        let x_idx = i % cols;
        let y_idx = i / cols;
        let x = ((x_idx as f32 + 0.5) * cell_w).clamp(0.0, width);
        let y = ((y_idx as f32 + 0.5) * cell_h).clamp(0.0, height);
        pos.push(Vec2::new(x, y));
    }

    pos
}
