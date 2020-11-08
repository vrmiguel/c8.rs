
use clap::{Arg, App, AppSettings};
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub struct Config {
	scale: u8,
	quirks: bool,
	delay: u8,
	filename: String
}

impl Config {
	pub fn new () -> Result<Config, std::io::Error> {
		let matches = App::new("lascaoito")
			.settings(&[AppSettings::ColoredHelp])
            .after_help("If you find any bugs, please file an issue at github.com/vrmiguel/lascaoito.")
            .version_message("Display version information.")
			.version("0.1.0")
			.author("Vinicius R. Miguel <vinicius.miguel at unifesp.br>")
			.about("CHIP-8 emulator")
			.arg(
				Arg::with_name("scale")
					.short("s")
					.long("scale")
					.value_name("SCALE")
					.help("Sets the video scale factor.")
					.takes_value(true))
			.arg(
				Arg::with_name("filename")
					.value_name("ROM")
					.help("The ROM file to be played.")
					.required(true)
					.takes_value(true))
			.arg(
				Arg::with_name("quirks")
					.short("q")
					.long("quirks")
					.help("Activate CPU quirks. May improve compatibility in some ROMs."))
			.arg(
				Arg::with_name("delay")
					.short("d")
					.long("delay")
					.help("The time between cycles, in milliseconds. Usually between 0 and 10.")
					.value_name("DELAY"))
			.get_matches();

		// This .unwrap() will always be Ok since filename is a required argument
		let rom_filename = matches.value_of("filename").unwrap();
		
		let cycle_delay = matches.value_of("delay").unwrap_or("1");
		let cycle_delay = cycle_delay.parse::<u8>();
		if cycle_delay.is_err() {
			return Err(Error::new(ErrorKind::Other, "Error: invalid argument passed on to -d/--delay."));
		}
		let cycle_delay = cycle_delay.unwrap();

		let scale_factor = matches.value_of("scale").unwrap_or("5");
		let scale_factor = scale_factor.parse::<u8>();
		if scale_factor.is_err() {
			return Err(Error::new(ErrorKind::Other, "Error: invalid argument passed on to -s/--scale."));
		}
		let scale_factor = scale_factor.unwrap();

		// TODO: read quirks

		Ok(Config {
			delay: cycle_delay, 
			scale: scale_factor, 
			filename: rom_filename.to_string(), 
			quirks: false
		})
	}
}