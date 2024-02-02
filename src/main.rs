//! This is a simple CHIP-8 interpreter based on this UWCS project:
//! <https://rs118.uwcs.co.uk/chip8.html>

#![feature(generic_arg_infer)]

mod interpreter;

fn main() {
    chip8_base::run(self::interpreter::Chip8Interpreter::new(700));
}
