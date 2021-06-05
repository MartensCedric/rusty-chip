use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render;
use sdl2::video::Window;
use std::env;
use std::error::Error;
use std::fs;
use std::time::Duration;
use crate::chip8::Chip8;

const PIXEL_SIZE: u32 = 10;

pub struct Config {
    pub rom_filename: String
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let rom_filename = args[1].clone();

        Ok(Config {
            rom_filename
        })
    }
}

fn index_to_point(index: i32) -> Point {
    Point::new((index % 64) * PIXEL_SIZE as i32, (index / 32) * PIXEL_SIZE as i32)
}

fn set_grid_index_color(canvas: &mut render::WindowCanvas, index: i32, alpha: u8) {
    canvas.set_draw_color(Color::RGBA(alpha, alpha, alpha, 255));
    let point: Point = index_to_point(index);
    match canvas.fill_rect(Rect::new(point.x, point.y, PIXEL_SIZE, PIXEL_SIZE)) {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Started rusty_chip!");
    let chip8: Chip8 = Chip8::new();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Rusty Chip",
            64 * PIXEL_SIZE,
            32 * PIXEL_SIZE,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    // let contents = fs::read_to_string(config.rom_filename)?;
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for (index, alpha) in chip8.gfx.iter().enumerate() {
            set_grid_index_color(&mut canvas, index as i32, *alpha);
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
