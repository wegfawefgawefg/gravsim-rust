mod config;
mod state;
mod types;

use config::{WINDOW_HEIGHT, WINDOW_WIDTH};
use state::GpuState;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
#[cfg(target_os = "linux")]
use winit::platform::x11::{WindowBuilderExtX11, XWindowType};
use winit::window::Window;
use winit::window::WindowBuilder;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("failed to create event loop");
    let builder = WindowBuilder::new()
        .with_title("gravsim wgpu")
        .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let window = configure_window_builder(builder)
        .build(&event_loop)
        .expect("failed to create window");
    center_window_on_primary_monitor(&event_loop, &window);

    let window_ref: &'static winit::window::Window = Box::leak(Box::new(window));
    let mut state = pollster::block_on(GpuState::new(window_ref));

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent { window_id, event } if window_id == window_ref.id() => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(size) => state.resize(size),
                WindowEvent::RedrawRequested => match state.render(window_ref) {
                    Ok(()) => {}
                    Err(wgpu::SurfaceError::Lost) => state.recover_surface(),
                    Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                    Err(wgpu::SurfaceError::Outdated)
                    | Err(wgpu::SurfaceError::Timeout) => {}
                },
                WindowEvent::CursorMoved { position, .. } => {
                    state.set_mouse(position.x as f32, position.y as f32)
                }
                _ => {}
            },
            Event::AboutToWait => window_ref.request_redraw(),
            _ => {}
        })
        .expect("event loop error");
}

fn center_window_on_primary_monitor(
    event_loop: &EventLoop<()>,
    window: &Window,
) {
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
