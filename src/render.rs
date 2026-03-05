use glam::Vec2;
use raylib::prelude::*;

use crate::config::{DRAW_BUDGET, FADE_AMOUNT, PIXEL_BRIGHTNESS, WINDOW_DIMS};

pub fn draw(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    positions: &[Vec2],
    texture: &mut RenderTexture2D,
    draw_offset: usize,
    mouse_pos: Vec2,
    perf_overlay: &str,
    show_overlay: bool,
) {
    let mut d = rl.begin_drawing(thread);
    let mut texture_d = d.begin_texture_mode(thread, texture);

    texture_d.draw_rectangle(
        0,
        0,
        WINDOW_DIMS.x,
        WINDOW_DIMS.y,
        Color::new(0, 0, 0, FADE_AMOUNT),
    );

    const PIXEL_COLOR: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: PIXEL_BRIGHTNESS,
    };

    let budget = DRAW_BUDGET.min(positions.len());
    let first_span = (positions.len() - draw_offset).min(budget);
    for pos in &positions[draw_offset..draw_offset + first_span] {
        texture_d.draw_pixel(pos.x as i32, pos.y as i32, PIXEL_COLOR);
    }
    for pos in &positions[..budget - first_span] {
        texture_d.draw_pixel(pos.x as i32, pos.y as i32, PIXEL_COLOR);
    }

    drop(texture_d);

    d.draw_texture_rec(
        texture,
        Rectangle::new(0.0, 0.0, WINDOW_DIMS.x as f32, -WINDOW_DIMS.y as f32),
        Vector2::new(0.0, 0.0),
        Color::WHITE,
    );

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
