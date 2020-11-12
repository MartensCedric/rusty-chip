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

    println!("Hello, world!");
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_test() {
        assert_eq!(3, 3);
    }
}
