use glam::Vec2;
use rand::{thread_rng, Rng};

use crate::config::{NUM_BODIES, WINDOW_CENTER};

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
