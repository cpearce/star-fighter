// Copyright 2018 Chris Pearce
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate rand;
extern crate sdl2;

use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::Window;
use std::cmp;

const BLOCK_SIZE: u32 = 5;
const MONSTER_SPAWN_RATE: f64 = 0.5;
const MISSILE_SPEED: i32 = 7;
const PLAYER_SPEED: i32 = 5;
const MONSTER_SPEED: i32 = 3;
const FIRE_COOLDOWN_MS: u32 = 250;
const STAR_SPAWN_PERIOD: u32 = 250;
const BIG_STAR_SPAWN_RATE: f64 = 0.1;
const BIG_STAR_SPEED: i32 = 2;
const LITTLE_STAR_SPAWN_RATE: f64 = 0.25;
const LITTLE_STAR_SPEED: i32 = 1;

fn player_sprite<'a, T>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<T>,
) -> Texture<'a> {
    let mut texture = texture_creator
        .create_texture_target(
            Some(PixelFormatEnum::RGBA8888),
            BLOCK_SIZE * 3,
            BLOCK_SIZE * 7,
        )
        .unwrap();

    let yellow = Color::RGBA(255, 255, 0, 255);
    let red = Color::RGBA(255, 0, 0, 255);
    let orange = Color::RGBA(255, 128, 0, 255);
    let pink = Color::RGBA(255, 0, 255, 255);
    let blue = Color::RGBA(0, 128, 255, 255);

    let pixels = [
        (1, 0, pink),
        (0, 1, pink),
        (1, 1, blue),
        (2, 1, pink),
        (0, 2, blue),
        (1, 2, red),
        (2, 2, blue),
        (0, 3, pink),
        (1, 3, pink),
        (2, 3, pink),
        (0, 4, pink),
        (2, 4, pink),
        (0, 5, yellow),
        (2, 5, yellow),
        (0, 6, orange),
        (2, 6, orange),
    ];

    canvas
        .with_texture_canvas(&mut texture, |texture_canvas| {
            for (dx, dy, color) in &pixels {
                let x = dx * BLOCK_SIZE as i32;
                let y = dy * BLOCK_SIZE as i32;
                texture_canvas.set_draw_color(*color);
                texture_canvas
                    .fill_rect(Rect::new(x, y, BLOCK_SIZE as u32, BLOCK_SIZE as u32))
                    .unwrap();
            }
        })
        .unwrap();

    texture
}

fn monster_sprite<'a, T>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<T>,
) -> Texture<'a> {
    // Monsters are bigger than the player!
    let scale = 2;
    let mut texture = texture_creator
        .create_texture_target(
            Some(PixelFormatEnum::RGBA8888),
            scale * BLOCK_SIZE * 3,
            scale * BLOCK_SIZE * 5,
        )
        .unwrap();

    let yellow = Color::RGBA(255, 255, 0, 255);
    let red = Color::RGBA(255, 0, 0, 255);
    let orange = Color::RGBA(255, 128, 0, 255);
    let green = Color::RGBA(0, 255, 0, 255);

    let pixels = [
        (0, 0, yellow),
        (2, 0, yellow),
        (0, 1, red),
        (2, 1, red),
        (0, 2, yellow),
        (1, 2, green),
        (2, 2, yellow),
        (1, 3, orange),
        (1, 4, orange),
    ];

    canvas
        .with_texture_canvas(&mut texture, |texture_canvas| {
            for (dx, dy, color) in &pixels {
                let x = (dx * scale * BLOCK_SIZE) as i32;
                let y = (dy * scale * BLOCK_SIZE) as i32;
                texture_canvas.set_draw_color(*color);
                texture_canvas
                    .fill_rect(Rect::new(
                        x,
                        y,
                        scale * BLOCK_SIZE as u32,
                        scale * BLOCK_SIZE as u32,
                    ))
                    .unwrap();
            }
        })
        .unwrap();

    texture
}

fn missile_sprite<'a, T>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<T>,
) -> Texture<'a> {
    let mut texture = texture_creator
        .create_texture_target(
            Some(PixelFormatEnum::RGBA8888),
            BLOCK_SIZE * 1,
            BLOCK_SIZE * 3,
        )
        .unwrap();

    let white = Color::RGBA(255, 255, 255, 255);
    let blue = Color::RGBA(0, 0, 255, 255);

    let pixels = [(0, 0, white), (0, 1, white), (0, 2, blue)];

    canvas
        .with_texture_canvas(&mut texture, |texture_canvas| {
            for (dx, dy, color) in &pixels {
                let x = dx * BLOCK_SIZE as i32;
                let y = dy * BLOCK_SIZE as i32;
                texture_canvas.set_draw_color(*color);
                texture_canvas
                    .fill_rect(Rect::new(x, y, BLOCK_SIZE as u32, BLOCK_SIZE as u32))
                    .unwrap();
            }
        })
        .unwrap();

    texture
}

fn draw_big_star(canvas: &mut Canvas<Window>, x: i32, y: i32) {
    let white = Color::RGBA(222, 233, 252, 255);
    let blue = Color::RGBA(36, 68, 122, 255);
    let pixels = [
        (-1, 0, blue),
        (0, -1, blue),
        (0, 0, white),
        (1, 0, blue),
        (0, 1, blue),
    ];
    for (dx, dy, color) in &pixels {
        let x = x + dx * BLOCK_SIZE as i32;
        let y = y + dy * BLOCK_SIZE as i32;
        canvas.set_draw_color(*color);
        canvas
            .fill_rect(Rect::new(x, y, BLOCK_SIZE as u32, BLOCK_SIZE as u32))
            .unwrap();
    }
}

fn draw_little_star(canvas: &mut Canvas<Window>, x: i32, y: i32) {
    canvas.set_draw_color(Color::RGBA(0, 0, 255, 255));
    canvas
        .fill_rect(Rect::new(x, y, BLOCK_SIZE as u32, BLOCK_SIZE as u32))
        .unwrap();
}

fn centered_at(x: i32, y: i32, width: u32, height: u32) -> Rect {
    Rect::new(x - width as i32 / 2, y - height as i32 / 2, width, height)
}

#[derive(Copy, Clone)]
struct Sprite {
    x: i32,
    y: i32,
}

impl Sprite {
    pub fn new(x: i32, y: i32) -> Sprite {
        Sprite { x, y }
    }
}

fn texture_size(texture: &Texture) -> (u32, u32) {
    let query = texture.query();
    (query.width, query.height)
}

fn populate_stars(
    spawn_rate: f64,
    screen_width: u32,
    screen_height: u32,
    starfield_speed: i32,
    refresh_rate: i32,
) -> Vec<Sprite> {
    // Simulate starfield generator running for as long as it takes for
    // the starfield to traverse onto the screen.
    let frames = screen_height as i32 / starfield_speed;
    let ms = ((frames as f64 / refresh_rate as f64) * 1000.0) as i32;
    let generations = (ms as f64 / STAR_SPAWN_PERIOD as f64) as i32;
    let mut starfield = vec![];
    let mut rng = thread_rng();
    for _ in 0..generations {
        let r: f64 = rng.gen();
        if r < spawn_rate {
            starfield.push(Sprite::new(
                rng.gen_range(0, screen_width as i32),
                rng.gen_range(0, screen_height as i32),
            ));
        }
    }

    starfield
}

pub fn main() {
    let width = 800;
    let height = 600;
    let sdl_context = sdl2::init().unwrap();
    let mut timer_subsystem = sdl_context.timer().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Star Fighter Returns!", width, height)
        .position_centered()
        .build()
        .unwrap();

    let display_mode = window.display_mode().unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let player_texture = player_sprite(&mut canvas, &texture_creator);
    let (player_sprite_width, player_sprite_height) = texture_size(&player_texture);

    let monster_texture = monster_sprite(&mut canvas, &texture_creator);
    let (monster_sprite_width, monster_sprite_height) = texture_size(&monster_texture);

    let missile_texture = missile_sprite(&mut canvas, &texture_creator);
    let (missile_sprite_width, missile_sprite_height) = texture_size(&missile_texture);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut x: i32 = width as i32 / 2;
    let mut y: i32 = (height as i32 / 4) * 3;
    let mut left_key_pressed = false;
    let mut right_key_pressed = false;
    let mut up_key_pressed = false;
    let mut down_key_pressed = false;
    let mut space_key_pressed = false;

    let mut monsters: Vec<Sprite> = vec![];
    let mut missiles: Vec<Sprite> = vec![];
    let mut big_stars: Vec<Sprite> = populate_stars(
        BIG_STAR_SPAWN_RATE,
        width,
        height,
        BIG_STAR_SPEED,
        display_mode.refresh_rate,
    );
    let mut little_stars: Vec<Sprite> = populate_stars(
        LITTLE_STAR_SPAWN_RATE,
        width,
        height,
        LITTLE_STAR_SPEED,
        display_mode.refresh_rate,
    );
    let mut rng = thread_rng();

    let mut spawn_ticker = timer_subsystem.ticks();
    let mut last_fire_tick = timer_subsystem.ticks();
    let mut star_ticker = timer_subsystem.ticks();
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                // Left key
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    left_key_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    left_key_pressed = false;
                }

                // Right key
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    right_key_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    right_key_pressed = false;
                }

                // Up key
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    up_key_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    up_key_pressed = false;
                }

                // Down key
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    down_key_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    down_key_pressed = false;
                }

                // Space key
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    space_key_pressed = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    space_key_pressed = false;
                }

                _ => {}
            }
        }

        let now = timer_subsystem.ticks();

        // Spawn monsters.
        if now - spawn_ticker >= 500 {
            spawn_ticker = now;
            let r: f64 = rng.gen();
            if r < MONSTER_SPAWN_RATE {
                let monster_x =
                    rng.gen_range(monster_sprite_width, width - monster_sprite_width) as i32;
                monsters.push(Sprite::new(
                    monster_x,
                    -1 * monster_sprite_height as i32 / 2,
                ))
            }
        }

        // Spawn stars.
        if now - star_ticker >= STAR_SPAWN_PERIOD {
            star_ticker = now;
            let r: f64 = rng.gen();
            if r < BIG_STAR_SPAWN_RATE {
                big_stars.push(Sprite::new(rng.gen_range(0, width as i32), 0));
            }
            let r: f64 = rng.gen();
            if r < LITTLE_STAR_SPAWN_RATE {
                little_stars.push(Sprite::new(rng.gen_range(0, width as i32), 0));
            }
        }

        // Move player.
        if left_key_pressed {
            x = cmp::max(player_sprite_width as i32 / 2, x - PLAYER_SPEED);
        }
        if right_key_pressed {
            x = cmp::min((width - player_sprite_width / 2) as i32, x + PLAYER_SPEED);
        }
        if down_key_pressed {
            y = cmp::min((height - player_sprite_height / 2) as i32, y + PLAYER_SPEED);
        }
        if up_key_pressed {
            y = cmp::max(player_sprite_height as i32 / 2, y - PLAYER_SPEED);
        }

        // Fire missiles.
        if space_key_pressed && (now - last_fire_tick) > FIRE_COOLDOWN_MS {
            last_fire_tick = now;
            missiles.push(Sprite::new(x, y));
        }

        // Move missiles.
        for missile in &mut missiles {
            missile.y -= MISSILE_SPEED;
        }

        // Filter missiles, dropping those which hit monsters or move
        // offscreen, and drop the monsters which are hit too.
        missiles = missiles
            .into_iter()
            .filter(|&missile| {
                let missile_rect = centered_at(
                    missile.x,
                    missile.y,
                    missile_sprite_width,
                    missile_sprite_height,
                );
                let (hit, missed) = monsters.iter().partition(|monster| {
                    let monster_rect = centered_at(
                        monster.x,
                        monster.y,
                        monster_sprite_width,
                        monster_sprite_height,
                    );
                    monster_rect.intersection(missile_rect).is_some()
                });
                monsters = missed;
                hit.is_empty() && missile.y > -1 * (missile_sprite_height as i32)
            })
            .collect();

        // If the player flies into a monster, game over!
        let player_rect = centered_at(x, y, player_sprite_width, player_sprite_height);
        for monster in &mut monsters {
            monster.y += MONSTER_SPEED;
            let monster_rect = centered_at(
                monster.x,
                monster.y,
                monster_sprite_width,
                monster_sprite_height,
            );
            if player_rect.has_intersection(monster_rect) {
                break 'running;
            }
        }

        // Drop monsters that move offscreen.
        monsters = monsters
            .into_iter()
            .filter(|m| m.y < (height + monster_sprite_height) as i32)
            .collect();

        // Paint little stars.
        for star in &mut little_stars {
            star.y += LITTLE_STAR_SPEED;
            draw_little_star(&mut canvas, star.x, star.y);
        }
        // Drop offscreen little stars.
        little_stars = little_stars
            .into_iter()
            .filter(|s| s.y < (height + BLOCK_SIZE) as i32)
            .collect();

        // Paint big stars.
        for star in &mut big_stars {
            star.y += BIG_STAR_SPEED;
            draw_big_star(&mut canvas, star.x, star.y);
        }
        // Drop offscreen big stars.
        big_stars = big_stars
            .into_iter()
            .filter(|s| s.y < (height + 3 * BLOCK_SIZE) as i32)
            .collect();

        // Paint missiles.
        for missile in &missiles {
            canvas
                .copy(
                    &missile_texture,
                    None,
                    centered_at(
                        missile.x,
                        missile.y,
                        missile_sprite_width,
                        missile_sprite_height,
                    ),
                )
                .unwrap();
        }

        // Paint player.
        canvas
            .copy(
                &player_texture,
                None,
                centered_at(x, y, player_sprite_width, player_sprite_height),
            )
            .unwrap();

        // Paint monsters.
        for monster in &monsters {
            canvas
                .copy(
                    &monster_texture,
                    None,
                    centered_at(
                        monster.x,
                        monster.y,
                        monster_sprite_width,
                        monster_sprite_height,
                    ),
                )
                .unwrap();
        }

        canvas.present();
    }
}
