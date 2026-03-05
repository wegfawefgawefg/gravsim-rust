pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;
pub const NUM_PARTICLES: u32 = 2_000_000;
pub const G: f32 = 40.0;
pub const ENABLE_BOUNDS: bool = true;
pub const USE_MOUSE_GRAVITY: bool = true;
pub const WORKGROUP_SIZE: u32 = 256;
pub const BLOCK_ON_GPU_EACH_FRAME: bool = true;

pub fn gravity_target(mouse: [f32; 2], width: u32, height: u32) -> [f32; 2] {
    if USE_MOUSE_GRAVITY {
        mouse
    } else {
        [width as f32 * 0.5, height as f32 * 0.5]
    }
}
