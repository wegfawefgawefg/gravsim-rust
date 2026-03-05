mod app;
mod benchmark;
mod bodies;
mod config;
mod render;
mod sim;

use raylib::prelude::*;

use app::run_interactive;
use benchmark::{parse_benchmark_config, run_benchmark};
use bodies::make_bodies;
use config::WINDOW_DIMS;
use render::Renderer;

fn main() {
    let benchmark = parse_benchmark_config();
    let mut bodies = make_bodies();

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_DIMS.x, WINDOW_DIMS.y)
        .title("Space, the initial frontier.!")
        .build();

    center_window_on_current_monitor(&mut rl);

    let mut renderer = Renderer::new(&mut rl, &thread).unwrap();

    if let Some(config) = benchmark {
        run_benchmark(&mut rl, &thread, &mut bodies, &mut renderer, &config);
        return;
    }

    run_interactive(&mut rl, &thread, &mut bodies, &mut renderer);
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
