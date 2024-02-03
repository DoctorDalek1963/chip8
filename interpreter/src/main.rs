//! This is a simple CHIP-8 interpreter based on this UWCS project:
//! <https://rs118.uwcs.co.uk/chip8.html>

#![feature(generic_arg_infer)]

mod interpreter;

use std::fs;

use clap::Parser;

/// Execute a ROM with a simple CHIP-8 interpreter.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// The filename of the ROM to execute.
    rom: String,

    /// The frequency of the interpreter's clock, measured in Hz.
    #[arg(long, short, default_value_t = 700.0)]
    frequency: f32,
}

fn main() {
    let args = Args::parse();

    let rom = match fs::read(args.rom) {
        Ok(data) => data,
        Err(e) => panic!("Failed to read file: {e:?}"),
    };

    chip8_base::run(self::interpreter::Chip8Interpreter::new(
        &rom,
        args.frequency,
    ));
}
