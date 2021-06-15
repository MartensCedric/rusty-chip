use crate::chip8::Chip8;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

const PIXEL_SIZE: u32 = 10;

pub struct Config {
    pub cartridge_rom_filename: String,
    pub console_rom_filename: String,
}
impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let cartridge_rom_filename = args[1].clone();

        Ok(Config {
            cartridge_rom_filename: cartridge_rom_filename,
            console_rom_filename: String::from("console_rom.dat"),
        })
    }
}

fn index_to_point(index: i32) -> Point {
    Point::new(
        (index % 64) * PIXEL_SIZE as i32,
        (index / 64) * PIXEL_SIZE as i32,
    )
}

fn set_grid_index_color(canvas: &mut render::WindowCanvas, index: i32, alpha: u8) {
    canvas.set_draw_color(Color::RGBA(alpha, alpha, alpha, 255));
    let point: Point = index_to_point(index);
    match canvas.fill_rect(Rect::new(point.x, point.y, PIXEL_SIZE, PIXEL_SIZE)) {
        Err(e) => println!("{:?}", e),
        _ => (),
    }
}

fn get_file_as_byte_vec(filename: &str) -> Vec<u8> {
    let mut f = File::open(&filename).expect(&format!("File named {} was not found!", filename));
    let metadata = fs::metadata(&filename).expect(&format!(
        "Unable to read metadata for file named {}",
        filename
    ));
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Started rusty_chip!");

    let mut chip8: Chip8 = Chip8::new();
    let console_rom: Vec<u8> = get_file_as_byte_vec(&config.console_rom_filename);
    chip8.init_memory(console_rom.iter().as_ref(), 0x0);

    let cartridge_rom: Vec<u8> = get_file_as_byte_vec(&config.cartridge_rom_filename);
    chip8.init_memory(cartridge_rom.iter().as_ref(), 0x200);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Rusty Chip", 64 * PIXEL_SIZE, 32 * PIXEL_SIZE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    chip8.key_states = 0;

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let is_ticking: bool = chip8.wait_key_state & 0xF0 == 0xF0;
        if is_ticking {
            chip8.decrement_timers();
        }

        for event in event_pump.poll_iter() {
            // http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.3
            // TODO: Take this logic out in another function, find a way to clean this
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Num0),
                    ..
                } => chip8.key_states |= 0x8000,
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.key_states |= 0x4000,
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.key_states |= 0x2000,
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.key_states |= 0x1000,
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.key_states |= 0x0800,
                Event::KeyDown {
                    keycode: Some(Keycode::Num5),
                    ..
                } => chip8.key_states |= 0x0400,
                Event::KeyDown {
                    keycode: Some(Keycode::Num6),
                    ..
                } => chip8.key_states |= 0x0200,
                Event::KeyDown {
                    keycode: Some(Keycode::Num7),
                    ..
                } => chip8.key_states |= 0x0100,
                Event::KeyDown {
                    keycode: Some(Keycode::Num8),
                    ..
                } => chip8.key_states |= 0x0080,
                Event::KeyDown {
                    keycode: Some(Keycode::Num9),
                    ..
                } => chip8.key_states |= 0x0040,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.key_states |= 0x0020,
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => chip8.key_states |= 0x0010,
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.key_states |= 0x0008,
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.key_states |= 0x0004,
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.key_states |= 0x0002,
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.key_states |= 0x0001,
                Event::KeyUp {
                    keycode: Some(Keycode::Num0),
                    ..
                } => chip8.key_states &= !0x8000,
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => chip8.key_states &= !0x4000,
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    ..
                } => chip8.key_states &= !0x2000,
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    ..
                } => chip8.key_states &= !0x1000,
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    ..
                } => chip8.key_states &= !0x0800,
                Event::KeyUp {
                    keycode: Some(Keycode::Num5),
                    ..
                } => chip8.key_states &= !0x0400,
                Event::KeyUp {
                    keycode: Some(Keycode::Num6),
                    ..
                } => chip8.key_states &= !0x0200,
                Event::KeyUp {
                    keycode: Some(Keycode::Num7),
                    ..
                } => chip8.key_states &= !0x0100,
                Event::KeyUp {
                    keycode: Some(Keycode::Num8),
                    ..
                } => chip8.key_states &= !0x0080,
                Event::KeyUp {
                    keycode: Some(Keycode::Num9),
                    ..
                } => chip8.key_states &= !0x0040,
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => chip8.key_states &= !0x0020,
                Event::KeyUp {
                    keycode: Some(Keycode::B),
                    ..
                } => chip8.key_states &= !0x0010,
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => chip8.key_states &= !0x0008,
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => chip8.key_states &= !0x0004,
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => chip8.key_states &= !0x0002,
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    ..
                } => chip8.key_states &= !0x0001,
                _ => {}
            }
        }

        if !is_ticking {
            for i in 0..16 {
                if (chip8.key_states >> (15 - i)) & 1 == 1 {
                    chip8.cpu_registers[chip8.wait_key_state as usize] = i as u8;
                    chip8.wait_key_state = 0xF0;
                }
            }
        }

        if is_ticking {
            chip8.fetch_cycle();
        }

        for (index, alpha) in chip8.gfx.iter().enumerate() {
            set_grid_index_color(&mut canvas, index as i32, *alpha);
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 1000));
    }
    Ok(())
}
