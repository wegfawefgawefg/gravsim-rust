use glam::Vec2;
use rayon::prelude::*;

use crate::bodies::Bodies;
use crate::config::{CHAIN_FIXED_SPEED, ENABLE_BOUNDS, WINDOW_DIMS};

pub fn step_chain(bodies: &mut Bodies, prev_pos: &mut [Vec2]) {
    prev_pos.copy_from_slice(&bodies.pos);
    let len = prev_pos.len();
    if len == 0 {
        return;
    }

    bodies
        .pos
        .par_iter_mut()
        .zip(bodies.vel.par_iter_mut())
        .enumerate()
        .for_each(|(i, (pos, vel))| {
            let target = prev_pos[(i + 1) % len];
            let delta = target - *pos;
            *vel = delta.normalize() * CHAIN_FIXED_SPEED;
            *pos += *vel;

            if ENABLE_BOUNDS {
                pos.x = pos.x.clamp(0.0, WINDOW_DIMS.x as f32);
                pos.y = pos.y.clamp(0.0, WINDOW_DIMS.y as f32);
            }
        });
}
