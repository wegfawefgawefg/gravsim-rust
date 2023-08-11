// define some body
// make some
// make them orbit the center?
// lol rayon

// show adam match cases lolurmotherobeast

use glam::{IVec2, Vec2};
use rand::{thread_rng, Rng};
use raylib::prelude::*;

use rayon::prelude::*;

const WINDOW_DIMS: IVec2 = IVec2 { x: 1280, y: 720 };
const WINDOW_CENTER: IVec2 = IVec2 {
    x: WINDOW_DIMS.x / 2,
    y: WINDOW_DIMS.y / 2,
};
const WINDOW_COLOR: Color = Color::WHITE;

const FRAMES_PER_SECOND: u32 = 144;
const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

const BODY_SIZE: i32 = 8;
const NUM_BODIES: i32 = 100000;

pub struct Body {
    pos: Vec2,
    vel: Vec2,
}

fn main() {
    let mut bodies: Vec<Body> = Vec::new();
    make_bodies(&mut bodies);

    let (mut rl, thread) = raylib::init()
        .size(WINDOW_DIMS.x, WINDOW_DIMS.y)
        .title("Space, the initial frontier.!")
        .build();

    let mut time_since_last_update = 0.0;
    while !rl.window_should_close() {
        let dt = rl.get_frame_time();
        time_since_last_update += dt;
        while time_since_last_update > TIMESTEP {
            time_since_last_update -= TIMESTEP;
            step(&mut bodies);
        }
        draw(&mut rl, &thread, &bodies);
    }
}

pub fn make_bodies(bodies: &mut Vec<Body>) {
    const INITIAL_VEL_MAX: f32 = 2.0;
    let spawn_start_p = WINDOW_CENTER.as_vec2() - WINDOW_CENTER.as_vec2() / 2.0;
    let spawn_end_p = WINDOW_CENTER.as_vec2() + WINDOW_CENTER.as_vec2() / 2.0;

    let mut rng = thread_rng();
    for _ in 0..NUM_BODIES {
        let new_body = Body {
            pos: Vec2::new(
                // rng.gen_range(1.0..WINDOW_DIMS.x as f32),
                // rng.gen_range(1.0..WINDOW_DIMS.y as f32),
                rng.gen_range(spawn_start_p.x..spawn_end_p.x),
                rng.gen_range(spawn_start_p.y..spawn_end_p.y),
            ),
            vel: Vec2::new(
                rng.gen_range(-INITIAL_VEL_MAX..INITIAL_VEL_MAX),
                rng.gen_range(-INITIAL_VEL_MAX..INITIAL_VEL_MAX),
            ),
        };
        bodies.push(new_body);
    }
}

pub fn step(bodies: &mut Vec<Body>) {
    // (m1 * m2) / (m1.pos - m2.pos)^2
    const G: f32 = 10.0;
    // for b in bodies.iter_mut() {
    //     let delta = WINDOW_CENTER.as_vec2() - b.pos;
    //     let dir = delta.normalize();
    //     let dist = delta.length();
    //     if dist > 2.0 {
    //         let f_mag = G * (1.0 / ((dist) + 0.0000000001));
    //         let f = dir * f_mag;
    //         b.vel += f;
    //     }
    //     b.pos += b.vel;
    // }
    bodies.par_iter_mut().for_each(|b| {
        let delta = WINDOW_CENTER.as_vec2() - b.pos;
        let dir = delta.normalize();
        let dist = delta.length();
        if dist > 2.0 {
            let f_mag = G * (1.0 / ((dist) + 0.0000000001));
            let f = dir * f_mag;
            b.vel += f;
        }
        b.pos += b.vel;
    });
}

pub fn draw(rl: &mut RaylibHandle, thread: &RaylibThread, bodies: &Vec<Body>) {
    let mut d = rl.begin_drawing(&thread);

    d.clear_background(Color::BLACK);
    // d.draw_rectangle(0, 0, WINDOW_DIMS.x, WINDOW_DIMS.y, Color::new(0, 0, 0, 255));

    for body in bodies.as_slice() {
        // d.draw_rectangle(
        //     (body.pos.x - BODY_SIZE as f32 / 2.0) as i32,
        //     (body.pos.y - BODY_SIZE as f32 / 2.0) as i32,
        //     BODY_SIZE,
        //     BODY_SIZE,
        //     Color::WHITE,
        // )
        d.draw_pixel(body.pos.x as i32, body.pos.y as i32, Color::WHITE);
    }
}
