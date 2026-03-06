pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;
pub const NUM_PARTICLES: u32 = 20_000_000;
pub const G: f32 = 8.0;
pub const ENABLE_BOUNDS: bool = true;
pub const USE_MOUSE_GRAVITY: bool = true;
pub const WORKGROUP_SIZE: u32 = 256;
pub const BLOCK_ON_GPU_EACH_FRAME: bool = true;
pub const DEFAULT_FADE_ENABLED: bool = false;
pub const FADE_ALPHA: f32 = 0.1;

#[derive(Clone, Copy, Debug)]
pub enum PresentModePreference {
    Immediate,
    Mailbox,
    Fifo,
}

pub const PRESENT_MODE_PREFERENCE: PresentModePreference = PresentModePreference::Immediate;

pub fn gravity_target(mouse: [f32; 2], width: u32, height: u32) -> [f32; 2] {
    if USE_MOUSE_GRAVITY {
        mouse
    } else {
        [width as f32 * 0.5, height as f32 * 0.5]
    }
}

pub fn pick_present_mode(caps: &wgpu::SurfaceCapabilities) -> wgpu::PresentMode {
    use wgpu::PresentMode;
    let requested = match PRESENT_MODE_PREFERENCE {
        PresentModePreference::Immediate => PresentMode::Immediate,
        PresentModePreference::Mailbox => PresentMode::Mailbox,
        PresentModePreference::Fifo => PresentMode::Fifo,
    };
    if caps.present_modes.contains(&requested) {
        return requested;
    }
    if caps.present_modes.contains(&PresentMode::Immediate) {
        return PresentMode::Immediate;
    }
    if caps.present_modes.contains(&PresentMode::Mailbox) {
        return PresentMode::Mailbox;
    }
    PresentMode::Fifo
}
