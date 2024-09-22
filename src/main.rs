use glam::{IVec2, Vec2};
use rand::{thread_rng, Rng};
use raylib::prelude::*;
use rayon::prelude::*;

const WINDOW_DIMS: IVec2 = IVec2 { x: 1280, y: 720 };
const WINDOW_CENTER: Vec2 = Vec2::new(WINDOW_DIMS.x as f32 / 2.0, WINDOW_DIMS.y as f32 / 2.0);
const FRAMES_PER_SECOND: u32 = 144;
const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;
const NUM_BODIES: usize = 8_000_000;
const G: f32 = 8.0;
const DRAW_BUDGET: usize = 200_000; // Number of bodies to draw per frame
const FADE_AMOUNT: u8 = 24; // Amount to subtract from alpha each frame
const PIXEL_BRIGHTNESS: u8 = 20; // Brightness of pixels

#[derive(Clone, Copy)]
pub struct Body {
    pos: Vec2,
    vel: Vec2,
}

fn main() {
    let mut bodies = make_bodies();
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_DIMS.x, WINDOW_DIMS.y)
        .title("Space, the initial frontier.!")
        .build();

    rl.set_window_position(200, 500);
    let mut time_since_last_update = 0.0;
    let mut texture = rl
        .load_render_texture(WINDOW_DIMS.x as u32, WINDOW_DIMS.y as u32)
        .unwrap();

    let mut draw_offset = 0;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_since_last_update += dt;

        let mouse_pos = rl.get_mouse_position();
        let mouse_pos = Vec2::new(mouse_pos.x, mouse_pos.y);

        while time_since_last_update > TIMESTEP {
            time_since_last_update -= TIMESTEP;
            step(&mut bodies, mouse_pos);
        }

        draw(
            &mut rl,
            &thread,
            &bodies,
            &mut texture,
            draw_offset,
            mouse_pos,
        );

        // Update draw_offset for next frame
        draw_offset = (draw_offset + DRAW_BUDGET) % NUM_BODIES;
    }
}

fn make_bodies() -> Vec<Body> {
    const INITIAL_VEL_MAX: f32 = 2.0;
    let spawn_start_p = WINDOW_CENTER - WINDOW_CENTER / 2.0;
    let spawn_end_p = WINDOW_CENTER + WINDOW_CENTER / 8.0;
    let mut rng = thread_rng();

    (0..NUM_BODIES)
        .map(|_| Body {
            pos: Vec2::new(
                rng.gen_range(spawn_start_p.x..spawn_end_p.x),
                rng.gen_range(spawn_start_p.y..spawn_end_p.y),
            ),
            vel: Vec2::new(
                rng.gen_range(-INITIAL_VEL_MAX..INITIAL_VEL_MAX),
                rng.gen_range(-INITIAL_VEL_MAX..INITIAL_VEL_MAX),
            ),
        })
        .collect()
}

fn step(bodies: &mut Vec<Body>, mouse_pos: Vec2) {
    bodies.par_iter_mut().for_each(|b| {
        let delta = mouse_pos - b.pos;
        let dir = delta.normalize();
        let dist = delta.length();
        if dist > 2.0 {
            let f_mag = G / (dist + 0.0000000001);
            let f = dir * f_mag;
            b.vel += f;
        }
        b.pos += b.vel;

        // clamp pos to screen
        b.pos.x = b.pos.x.max(0.0).min(WINDOW_DIMS.x as f32);
        b.pos.y = b.pos.y.max(0.0).min(WINDOW_DIMS.y as f32);
    });
}

fn draw(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    bodies: &[Body],
    texture: &mut RenderTexture2D,
    draw_offset: usize,
    mouse_pos: Vec2,
) {
    let mut d = rl.begin_drawing(thread);

    // Begin texture mode
    let mut texture_d = d.begin_texture_mode(thread, texture);

    // Draw a semi-transparent black rectangle to create fade effect
    texture_d.draw_rectangle(
        0,
        0,
        WINDOW_DIMS.x,
        WINDOW_DIMS.y,
        Color::new(0, 0, 0, FADE_AMOUNT),
    );

    // Draw bodies to texture, only up to DRAW_BUDGET
    const PIXEL_COLOR: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: PIXEL_BRIGHTNESS,
    };
    for i in 0..DRAW_BUDGET {
        let index = (draw_offset + i) % NUM_BODIES;
        let body = &bodies[index];
        texture_d.draw_pixel(body.pos.x as i32, body.pos.y as i32, PIXEL_COLOR);
    }

    // End texture mode
    drop(texture_d);

    // Draw the texture to the screen
    d.draw_texture_rec(
        texture,
        Rectangle::new(0.0, 0.0, WINDOW_DIMS.x as f32, -WINDOW_DIMS.y as f32),
        Vector2::new(0.0, 0.0),
        Color::WHITE,
    );

    // Display FPS and draw information
    d.draw_text(&format!("FPS: {}", d.get_fps()), 10, 10, 20, Color::LIME);
    d.draw_text(
        &format!("Drawing {}/{} bodies", DRAW_BUDGET, NUM_BODIES),
        10,
        30,
        20,
        Color::LIME,
    );

    // Draw a circle at the mouse position
    d.draw_circle(mouse_pos.x as i32, mouse_pos.y as i32, 5.0, Color::RED);
}
