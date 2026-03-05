use glam::{IVec2, Vec2};

pub const WINDOW_DIMS: IVec2 = IVec2 { x: 1280, y: 720 };
pub const WINDOW_CENTER: Vec2 = Vec2::new(WINDOW_DIMS.x as f32 / 2.0, WINDOW_DIMS.y as f32 / 2.0);
pub const FRAMES_PER_SECOND: u32 = 144;
pub const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;
pub const NUM_BODIES: usize = 10_000_000;
pub const G: f32 = 40.0;
pub const DRAW_BUDGET: usize = 200_000;
pub const FADE_AMOUNT: u8 = 24;
pub const PIXEL_BRIGHTNESS: u8 = 20;
pub const DEFAULT_STEP_CHUNK_SIZE: usize = 16_384;
