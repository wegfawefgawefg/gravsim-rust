#[path = "../common/bodies.rs"]
mod bodies;
#[path = "../common/config.rs"]
mod config;
#[path = "../common/render.rs"]
mod render;
mod sim;

use glam::Vec2;
use raylib::prelude::*;
use std::time::{Duration, Instant};
use std::{env, process};

use bodies::make_chain_bodies;
use config::{DRAW_BUDGET, TIMESTEP, WINDOW_DIMS};
use render::{RenderMode, Renderer};
use sim::step_chain;

fn main() {
    let seed = parse_seed_arg();
    let mut bodies = make_chain_bodies(seed);
    let mut prev_pos = vec![Vec2::ZERO; bodies.pos.len()];

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_DIMS.x, WINDOW_DIMS.y)
        .title("Chain CPU")
        .build();

    center_window_on_current_monitor(&mut rl);

    let mut renderer = Renderer::new(&mut rl, &thread, RenderMode::Rgba).unwrap();
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

        while time_since_last_update > TIMESTEP {
            time_since_last_update -= TIMESTEP;
            let step_start = Instant::now();
            step_chain(&mut bodies, &mut prev_pos);
            step_time_total += step_start.elapsed();
            steps_sampled += 1;
        }

        let draw_start = Instant::now();
        let mouse = rl.get_mouse_position();
        renderer.draw_positions(
            &mut rl,
            &thread,
            &bodies.pos,
            draw_offset,
            Vec2::new(mouse.x, mouse.y),
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
                "chain cpu step: {:.2}ms ({:.1} steps/s) draw: {:.2}ms/frame",
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

fn parse_seed_arg() -> Option<u64> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--seed" | "--spawn-seed" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for {arg}");
                    process::exit(2);
                });
                return Some(value.parse::<u64>().unwrap_or_else(|_| {
                    eprintln!("invalid seed value: {value}");
                    process::exit(2);
                }));
            }
            "--help" | "-h" => {
                println!("usage: chain_cpu [--seed <u64>]");
                process::exit(0);
            }
            _ => {}
        }
    }
    None
}

fn center_window_on_current_monitor(rl: &mut RaylibHandle) {
    let monitor = raylib::core::window::get_current_monitor();
    let monitor_width = raylib::core::window::get_monitor_width(monitor);
    let monitor_height = raylib::core::window::get_monitor_height(monitor);
    let monitor_pos = unsafe { raylib::ffi::GetMonitorPosition(monitor) };

    let centered_x = monitor_pos.x as i32 + (monitor_width - WINDOW_DIMS.x).max(0) / 2;
    let centered_y = monitor_pos.y as i32 + (monitor_height - WINDOW_DIMS.y).max(0) / 2;
    rl.set_window_position(centered_x, centered_y);
}
