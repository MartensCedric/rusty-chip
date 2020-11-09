use rusty_chip::Config;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem with arguments: {}", err);
        process::exit(1);
    });

    println!("Hello, world!");
}
