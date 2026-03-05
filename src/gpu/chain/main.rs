mod camera;
mod config;
mod state;
#[path = "../common/types.rs"]
mod types;

use config::{WINDOW_HEIGHT, WINDOW_WIDTH};
use state::ChainGpuState;
use std::{env, process};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
#[cfg(target_os = "linux")]
use winit::platform::x11::{WindowBuilderExtX11, XWindowType};
use winit::window::{Window, WindowBuilder};

fn main() {
    env_logger::init();
    let seed = parse_seed_arg();

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let builder = WindowBuilder::new()
        .with_title("Chain GPU")
        .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let window = configure_window_builder(builder)
        .build(&event_loop)
        .expect("failed to create window");
    center_window_on_primary_monitor(&event_loop, &window);

    let window_ref: &'static winit::window::Window = Box::leak(Box::new(window));
    let mut state = pollster::block_on(ChainGpuState::new(window_ref, seed));

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent { window_id, event } if window_id == window_ref.id() => {
                match event {
                    WindowEvent::CloseRequested => target.exit(),
                    WindowEvent::Resized(size) => state.resize(size),
                    WindowEvent::RedrawRequested => match state.render(window_ref) {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost) => state.recover_surface(),
                        Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                        Err(wgpu::SurfaceError::Outdated) | Err(wgpu::SurfaceError::Timeout) => {}
                    },
                    WindowEvent::KeyboardInput { event, .. } => {
                        if is_escape_pressed(&event) {
                            target.exit();
                        }
                    }
                    WindowEvent::MouseInput {
                        state: button_state,
                        button: MouseButton::Right,
                        ..
                    } => state.set_pan_active(button_state == ElementState::Pressed),
                    WindowEvent::CursorMoved { position, .. } => {
                        state.on_cursor_moved(position.x as f32, position.y as f32)
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        state.zoom_by_scroll(scroll_steps(delta));
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => window_ref.request_redraw(),
            _ => {}
        })
        .expect("event loop error");
}

fn center_window_on_primary_monitor(event_loop: &EventLoop<()>, window: &Window) {
    let Some(monitor) = event_loop.primary_monitor() else {
        return;
    };
    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();

    let x = monitor_pos.x + (monitor_size.width as i32 - WINDOW_WIDTH as i32).max(0) / 2;
    let y = monitor_pos.y + (monitor_size.height as i32 - WINDOW_HEIGHT as i32).max(0) / 2;
    window.set_outer_position(PhysicalPosition::new(x, y));
}

fn configure_window_builder(builder: WindowBuilder) -> WindowBuilder {
    #[cfg(target_os = "linux")]
    {
        return builder
            .with_name("raylib", "raylib")
            .with_x11_window_type(vec![XWindowType::Dialog, XWindowType::Utility]);
    }

    #[cfg(not(target_os = "linux"))]
    {
        builder
    }
}

fn is_escape_pressed(event: &winit::event::KeyEvent) -> bool {
    event.state == ElementState::Pressed
        && matches!(event.physical_key, PhysicalKey::Code(KeyCode::Escape))
}

fn scroll_steps(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => y,
        MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 40.0,
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
                println!("usage: chain_gpu [--seed <u64>]");
                process::exit(0);
            }
            _ => {}
        }
    }
    None
}
