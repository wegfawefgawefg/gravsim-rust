use glam::Vec2;
use raylib::core::texture::{Image, RaylibTexture2D, Texture2D};
use raylib::prelude::*;

use crate::config::{DRAW_BUDGET, FADE_AMOUNT, PIXEL_BRIGHTNESS, WINDOW_DIMS};

pub struct Renderer {
    texture: Texture2D,
    pixels: Vec<u8>,
    width: usize,
    height: usize,
}

impl Renderer {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, String> {
        let image = Image::gen_image_color(WINDOW_DIMS.x, WINDOW_DIMS.y, Color::BLACK);
        let texture = rl.load_texture_from_image(thread, &image)?;
        let width = WINDOW_DIMS.x as usize;
        let height = WINDOW_DIMS.y as usize;
        let mut pixels = vec![0; width * height * 4];

        for px in pixels.chunks_exact_mut(4) {
            px[3] = 255;
        }

        Ok(Self {
            texture,
            pixels,
            width,
            height,
        })
    }

    pub fn draw(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        positions: &[Vec2],
        draw_offset: usize,
        mouse_pos: Vec2,
        perf_overlay: &str,
        show_overlay: bool,
    ) {
        self.fade_pixels();
        let budget = self.plot_positions(positions, draw_offset);
        self.texture.update_texture(&self.pixels);

        let mut d = rl.begin_drawing(thread);
        d.draw_texture(&self.texture, 0, 0, Color::WHITE);

        if show_overlay {
            d.draw_text(&format!("FPS: {}", d.get_fps()), 10, 10, 20, Color::LIME);
            d.draw_text(
                &format!("Drawing {}/{} bodies", budget, positions.len()),
                10,
                30,
                20,
                Color::LIME,
            );
            d.draw_text(perf_overlay, 10, 50, 20, Color::YELLOW);
        }

        d.draw_circle(mouse_pos.x as i32, mouse_pos.y as i32, 5.0, Color::RED);
    }

    fn fade_pixels(&mut self) {
        for px in self.pixels.chunks_exact_mut(4) {
            px[0] = px[0].saturating_sub(FADE_AMOUNT);
            px[1] = px[1].saturating_sub(FADE_AMOUNT);
            px[2] = px[2].saturating_sub(FADE_AMOUNT);
        }
    }

    fn plot_positions(&mut self, positions: &[Vec2], draw_offset: usize) -> usize {
        let budget = DRAW_BUDGET.min(positions.len());
        let first_span = (positions.len() - draw_offset).min(budget);

        for pos in &positions[draw_offset..draw_offset + first_span] {
            self.plot_pixel(pos.x as i32, pos.y as i32);
        }
        for pos in &positions[..budget - first_span] {
            self.plot_pixel(pos.x as i32, pos.y as i32);
        }

        budget
    }

    fn plot_pixel(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }

        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return;
        }

        let idx = (y * self.width + x) * 4;
        self.pixels[idx] = self.pixels[idx].saturating_add(PIXEL_BRIGHTNESS);
        self.pixels[idx + 1] = self.pixels[idx + 1].saturating_add(PIXEL_BRIGHTNESS);
        self.pixels[idx + 2] = self.pixels[idx + 2].saturating_add(PIXEL_BRIGHTNESS);
    }
}
