use glam::Vec2;
use rayon::prelude::*;

use crate::bodies::Bodies;
use crate::config::{ENABLE_BOUNDS, G, WINDOW_DIMS};

#[derive(Clone, Copy, Debug)]
pub enum StepKernel {
    Zip,
    Chunked { chunk_size: usize },
}

#[derive(Clone, Copy, Debug)]
pub struct DrawSelection {
    pub draw_offset: usize,
    pub draw_budget: usize,
    pub total_bodies: usize,
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

pub fn step_with_kernel_collect_draw_indices(
    bodies: &mut Bodies,
    mouse_pos: Vec2,
    kernel: StepKernel,
    draw: DrawSelection,
) -> Vec<u32> {
    if draw.draw_budget == 0 || draw.total_bodies == 0 {
        step_with_kernel(bodies, mouse_pos, kernel);
        return Vec::new();
    }

    match kernel {
        StepKernel::Zip => bodies
            .pos
            .par_iter_mut()
            .zip(bodies.vel.par_iter_mut())
            .enumerate()
            .fold(Vec::<u32>::new, |mut local, (i, (pos, vel))| {
                apply_step(pos, vel, mouse_pos);
                if draw.contains(i) {
                    if let Some(pixel_index) = pixel_index_for_pos(*pos) {
                        local.push(pixel_index);
                    }
                }
                local
            })
            .reduce(Vec::<u32>::new, |mut a, b| {
                a.extend(b);
                a
            }),
        StepKernel::Chunked { chunk_size } => {
            let chunk_size = chunk_size.max(1);
            bodies
                .pos
                .par_chunks_mut(chunk_size)
                .zip(bodies.vel.par_chunks_mut(chunk_size))
                .enumerate()
                .fold(
                    Vec::<u32>::new,
                    |mut local, (chunk_idx, (pos_chunk, vel_chunk))| {
                        let start = chunk_idx * chunk_size;
                        for (local_i, (pos, vel)) in
                            pos_chunk.iter_mut().zip(vel_chunk.iter_mut()).enumerate()
                        {
                            let global_i = start + local_i;
                            apply_step(pos, vel, mouse_pos);
                            if draw.contains(global_i) {
                                if let Some(pixel_index) = pixel_index_for_pos(*pos) {
                                    local.push(pixel_index);
                                }
                            }
                        }
                        local
                    },
                )
                .reduce(Vec::<u32>::new, |mut a, b| {
                    a.extend(b);
                    a
                })
        }
    }
}

pub fn step_kernel_label(kernel: StepKernel) -> String {
    match kernel {
        StepKernel::Zip => String::from("zip"),
        StepKernel::Chunked { chunk_size } => format!("chunked:{chunk_size}"),
    }
}

impl DrawSelection {
    fn contains(self, index: usize) -> bool {
        if self.draw_budget >= self.total_bodies {
            return true;
        }
        let start = self.draw_offset % self.total_bodies;
        let end = start + self.draw_budget;
        if end <= self.total_bodies {
            index >= start && index < end
        } else {
            index >= start || index < (end - self.total_bodies)
        }
    }
}

fn apply_step(pos: &mut Vec2, vel: &mut Vec2, mouse_pos: Vec2) {
    let delta = mouse_pos - *pos;
    let dist_sq = delta.length_squared();
    if dist_sq > 4.0 {
        *vel += delta * (G / dist_sq);
    }
    *pos += *vel;

    if ENABLE_BOUNDS {
        pos.x = pos.x.clamp(0.0, WINDOW_DIMS.x as f32);
        pos.y = pos.y.clamp(0.0, WINDOW_DIMS.y as f32);
    }
}

fn pixel_index_for_pos(pos: Vec2) -> Option<u32> {
    let x = pos.x as i32;
    let y = pos.y as i32;
    if x < 0 || y < 0 || x >= WINDOW_DIMS.x || y >= WINDOW_DIMS.y {
        return None;
    }
    Some((y as usize * WINDOW_DIMS.x as usize + x as usize) as u32)
}
