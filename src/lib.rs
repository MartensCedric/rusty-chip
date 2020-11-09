pub mod rusty_chip {

    use std::env;
    use std::error::Error;
    use std::fs;

    pub struct Config {
        pub rom_filename: String,
    }

    pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
        let contents = fs::read_to_string(config.rom_filename)?;
        println!("Started rusty_chip!");
        Ok(())
    }
}
