use std::env;
use std::error::Error;
use std::fs;

pub struct Config {
    pub rom_filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        }

        let rom_filename = args[1].clone();

        Ok(Config { rom_filename })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.rom_filename)?;
    println!("Started rusty_chip!");
    Ok(())
}
