use glam::Vec2;
use rayon::prelude::*;

use crate::bodies::Bodies;
use crate::config::{G, WINDOW_DIMS};

#[derive(Clone, Copy, Debug)]
pub enum StepKernel {
    Zip,
    Chunked { chunk_size: usize },
}

pub fn step_with_kernel(bodies: &mut Bodies, mouse_pos: Vec2, kernel: StepKernel) {
    match kernel {
        StepKernel::Zip => {
            bodies
                .pos
                .par_iter_mut()
                .zip(bodies.vel.par_iter_mut())
                .for_each(|(pos, vel)| apply_step(pos, vel, mouse_pos));
        }
        StepKernel::Chunked { chunk_size } => {
            let chunk_size = chunk_size.max(1);
            bodies
                .pos
                .par_chunks_mut(chunk_size)
                .zip(bodies.vel.par_chunks_mut(chunk_size))
                .for_each(|(pos_chunk, vel_chunk)| {
                    for (pos, vel) in pos_chunk.iter_mut().zip(vel_chunk.iter_mut()) {
                        apply_step(pos, vel, mouse_pos);
                    }
                });
        }
    }
}

pub fn step_kernel_label(kernel: StepKernel) -> String {
    match kernel {
        StepKernel::Zip => String::from("zip"),
        StepKernel::Chunked { chunk_size } => format!("chunked:{chunk_size}"),
    }
}

fn apply_step(pos: &mut Vec2, vel: &mut Vec2, mouse_pos: Vec2) {
    let delta = mouse_pos - *pos;
    let dist_sq = delta.length_squared();
    if dist_sq > 4.0 {
        *vel += delta * (G / dist_sq);
    }
    *pos += *vel;

    pos.x = pos.x.clamp(0.0, WINDOW_DIMS.x as f32);
    pos.y = pos.y.clamp(0.0, WINDOW_DIMS.y as f32);
}
