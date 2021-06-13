use std::env;
use std::process;

mod chip8;
mod chip8_sdl2_gui;
mod chip8_util;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = chip8_sdl2_gui::Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem with arguments: {}", err);
        process::exit(1);
    });

    match chip8_sdl2_gui::run(config) {
        Ok(_x) => {
            println!("Thank for playing!");
        }
        Err(x) => {
            println!("Error while running SDL2_GUI: {}", x);
        }
    }
}
