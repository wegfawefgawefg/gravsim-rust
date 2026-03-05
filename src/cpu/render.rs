use glam::Vec2;
use raylib::core::texture::{Image, RaylibTexture2D, Texture2D};
use raylib::prelude::*;

use crate::config::{DRAW_BUDGET, FADE_AMOUNT, PIXEL_BRIGHTNESS, WINDOW_DIMS};

#[derive(Clone, Copy, Debug)]
pub enum RenderMode {
    Rgba,
    Bitset,
}

pub struct Renderer {
    mode: RenderMode,
    texture: Texture2D,
    rgba_pixels: Vec<u8>,
    bit_pixels: Vec<u64>,
    width: usize,
    height: usize,
}

impl Renderer {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        mode: RenderMode,
    ) -> Result<Self, String> {
        let image = Image::gen_image_color(WINDOW_DIMS.x, WINDOW_DIMS.y, Color::BLACK);
        let texture = rl.load_texture_from_image(thread, &image)?;
        let width = WINDOW_DIMS.x as usize;
        let height = WINDOW_DIMS.y as usize;
        let mut rgba_pixels = vec![0; width * height * 4];

        for px in rgba_pixels.chunks_exact_mut(4) {
            px[3] = 255;
        }

        let bit_len = (width * height).div_ceil(64);
        let bit_pixels = vec![0_u64; bit_len];

        Ok(Self {
            mode,
            texture,
            rgba_pixels,
            bit_pixels,
            width,
            height,
        })
    }

    pub fn draw_positions(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        positions: &[Vec2],
        draw_offset: usize,
        mouse_pos: Vec2,
        perf_overlay: &str,
        show_overlay: bool,
    ) {
        self.begin_frame();
        let budget = self.plot_positions(positions, draw_offset);
        self.upload_texture();
        self.present(
            rl,
            thread,
            mouse_pos,
            perf_overlay,
            show_overlay,
            budget,
            positions.len(),
        );
    }

    pub fn draw_indices(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        pixel_indices: &[u32],
        mouse_pos: Vec2,
        perf_overlay: &str,
        show_overlay: bool,
        total_bodies: usize,
    ) {
        self.begin_frame();
        self.plot_linear_indices(pixel_indices);
        self.upload_texture();
        self.present(
            rl,
            thread,
            mouse_pos,
            perf_overlay,
            show_overlay,
            pixel_indices.len(),
            total_bodies,
        );
    }

    fn begin_frame(&mut self) {
        match self.mode {
            RenderMode::Rgba => self.fade_rgba(),
            RenderMode::Bitset => self.bit_pixels.fill(0),
        }
    }

    fn plot_positions(&mut self, positions: &[Vec2], draw_offset: usize) -> usize {
        let budget = DRAW_BUDGET.min(positions.len());
        let first_span = (positions.len() - draw_offset).min(budget);

        for pos in &positions[draw_offset..draw_offset + first_span] {
            self.plot_pixel_xy(pos.x as i32, pos.y as i32);
        }
        for pos in &positions[..budget - first_span] {
            self.plot_pixel_xy(pos.x as i32, pos.y as i32);
        }

        budget
    }

    fn plot_linear_indices(&mut self, pixel_indices: &[u32]) {
        for &pixel_index in pixel_indices {
            self.plot_pixel_idx(pixel_index as usize);
        }
    }

    fn plot_pixel_xy(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }

        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return;
        }

        self.plot_pixel_idx(y * self.width + x);
    }

    fn plot_pixel_idx(&mut self, pixel_idx: usize) {
        match self.mode {
            RenderMode::Rgba => {
                let idx = pixel_idx * 4;
                self.rgba_pixels[idx] = self.rgba_pixels[idx].saturating_add(PIXEL_BRIGHTNESS);
                self.rgba_pixels[idx + 1] =
                    self.rgba_pixels[idx + 1].saturating_add(PIXEL_BRIGHTNESS);
                self.rgba_pixels[idx + 2] =
                    self.rgba_pixels[idx + 2].saturating_add(PIXEL_BRIGHTNESS);
            }
            RenderMode::Bitset => {
                let word = pixel_idx / 64;
                let bit = pixel_idx % 64;
                self.bit_pixels[word] |= 1_u64 << bit;
            }
        }
    }

    fn fade_rgba(&mut self) {
        for px in self.rgba_pixels.chunks_exact_mut(4) {
            px[0] = px[0].saturating_sub(FADE_AMOUNT);
            px[1] = px[1].saturating_sub(FADE_AMOUNT);
            px[2] = px[2].saturating_sub(FADE_AMOUNT);
        }
    }

    fn upload_texture(&mut self) {
        match self.mode {
            RenderMode::Rgba => {
                self.texture.update_texture(&self.rgba_pixels);
            }
            RenderMode::Bitset => {
                self.bitset_to_rgba();
                self.texture.update_texture(&self.rgba_pixels);
            }
        }
    }

    fn bitset_to_rgba(&mut self) {
        for pixel_idx in 0..(self.width * self.height) {
            let word = pixel_idx / 64;
            let bit = pixel_idx % 64;
            let on = (self.bit_pixels[word] >> bit) & 1;
            let value = if on == 1 { 255 } else { 0 };

            let idx = pixel_idx * 4;
            self.rgba_pixels[idx] = value;
            self.rgba_pixels[idx + 1] = value;
            self.rgba_pixels[idx + 2] = value;
            self.rgba_pixels[idx + 3] = 255;
        }
    }

    fn present(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        mouse_pos: Vec2,
        perf_overlay: &str,
        show_overlay: bool,
        drawn_count: usize,
        total_bodies: usize,
    ) {
        let mut d = rl.begin_drawing(thread);
        d.draw_texture(&self.texture, 0, 0, Color::WHITE);

        if show_overlay {
            d.draw_text(&format!("FPS: {}", d.get_fps()), 10, 10, 20, Color::LIME);
            d.draw_text(
                &format!("Drawing {}/{} bodies", drawn_count, total_bodies),
                10,
                30,
                20,
                Color::LIME,
            );
            d.draw_text(perf_overlay, 10, 50, 20, Color::YELLOW);
            d.draw_text(
                &format!("render mode: {}", render_mode_label(self.mode)),
                10,
                70,
                20,
                Color::SKYBLUE,
            );
        }

        d.draw_circle(mouse_pos.x as i32, mouse_pos.y as i32, 5.0, Color::RED);
    }
}

pub fn render_mode_label(mode: RenderMode) -> &'static str {
    match mode {
        RenderMode::Rgba => "rgba",
        RenderMode::Bitset => "bitset",
    }
}
