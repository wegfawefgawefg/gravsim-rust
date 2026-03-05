use glam::Vec2;
use raylib::prelude::*;
use std::time::{Duration, Instant};

use crate::bodies::Bodies;
use crate::config::{DRAW_BUDGET, TIMESTEP};
use crate::render::Renderer;
use crate::sim::{step_with_kernel, StepKernel};

pub fn run_interactive(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    bodies: &mut Bodies,
    renderer: &mut Renderer,
) {
    let mut time_since_last_update = 0.0;
    let mut draw_offset = 0;
    let mut step_time_total = Duration::ZERO;
    let mut draw_time_total = Duration::ZERO;
    let mut steps_sampled: usize = 0;
    let mut frames_sampled: usize = 0;
    let mut profile_window_start = Instant::now();
    let mut perf_overlay = String::from("step: collecting... draw: collecting...");

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_since_last_update += dt;

        let mouse_pos = rl.get_mouse_position();
        let mouse_pos = Vec2::new(mouse_pos.x, mouse_pos.y);

        while time_since_last_update > TIMESTEP {
            time_since_last_update -= TIMESTEP;
            let step_start = Instant::now();
            // Keep zip kernel for interactive; benchmark mode can test chunked variants.
            step_with_kernel(bodies, mouse_pos, StepKernel::Zip);
            step_time_total += step_start.elapsed();
            steps_sampled += 1;
        }

        let draw_start = Instant::now();
        renderer.draw_positions(
            rl,
            thread,
            &bodies.pos,
            draw_offset,
            mouse_pos,
            &perf_overlay,
            true,
        );
        draw_time_total += draw_start.elapsed();
        frames_sampled += 1;

        let window_elapsed = profile_window_start.elapsed();
        if window_elapsed >= Duration::from_secs(1) {
            let step_ms = if steps_sampled > 0 {
                (step_time_total.as_secs_f64() * 1000.0) / steps_sampled as f64
            } else {
                0.0
            };
            let draw_ms = if frames_sampled > 0 {
                (draw_time_total.as_secs_f64() * 1000.0) / frames_sampled as f64
            } else {
                0.0
            };
            let steps_per_sec = steps_sampled as f64 / window_elapsed.as_secs_f64();
            perf_overlay = format!(
                "step: {:.2}ms/step ({:.1} steps/s) draw: {:.2}ms/frame",
                step_ms, steps_per_sec, draw_ms
            );

            step_time_total = Duration::ZERO;
            draw_time_total = Duration::ZERO;
            steps_sampled = 0;
            frames_sampled = 0;
            profile_window_start = Instant::now();
        }

        draw_offset = (draw_offset + DRAW_BUDGET) % bodies.pos.len();
    }
}
