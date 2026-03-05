use glam::Vec2;
use rayon::prelude::*;

use crate::bodies::Bodies;
use crate::config::{G, WINDOW_DIMS};

pub fn step(bodies: &mut Bodies, mouse_pos: Vec2) {
    bodies
        .pos
        .par_iter_mut()
        .zip(bodies.vel.par_iter_mut())
        .for_each(|(pos, vel)| {
            let delta = mouse_pos - *pos;
            let dist_sq = delta.length_squared();
            if dist_sq > 4.0 {
                *vel += delta * (G / dist_sq);
            }
            *pos += *vel;

            pos.x = pos.x.clamp(0.0, WINDOW_DIMS.x as f32);
            pos.y = pos.y.clamp(0.0, WINDOW_DIMS.y as f32);
        });
}
