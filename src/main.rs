use std::env;
use std::process;

mod chip8;
mod chip8_gui;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = chip8_gui::Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem with arguments: {}", err);
        process::exit(1);
    });

    chip8_gui::run(config);
}
