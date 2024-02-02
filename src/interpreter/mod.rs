//! This module contains the [`Interpreter`] type.

mod instruction;

use self::instruction::{decode, Instruction};
use chip8_base::{Display, Interpreter, Keys, Pixel};
use std::time::Duration;

/// A simple CHIP-8 interpreter.
///
/// See the CHIP-8 spec here: <http://devernay.free.fr/hacks/chip8/C8TECH10.HTM>.
#[derive(Clone, Copy, Debug)]
pub struct Chip8Interpreter {
    /// All the memory of the interpreter.
    memory: [u8; 4096],

    /// The stack, used to keep track of return addresses.
    stack: [u16; 16],

    /// General purpose registers V0, V1, ... VF.
    v_registers: [u8; 16],

    /// The `I` register, used to store memory addresses.
    memory_register: u16,

    /// The delay timer (DT) register.
    delay_timer: u8,

    /// The sound timer (ST) register.
    sound_timer: u8,

    /// The program counter. Points to the next instruction to execute.
    program_counter: u16,

    /// The stack pointer. Points to the top of the stack.
    stack_pointer: u8,

    /// The current display.
    display: Display,

    /// The speed of the interpreter.
    speed: Duration,
}

impl Chip8Interpreter {
    /// Create a new instance of the interpreter.
    ///
    /// The clock frequency is measure in Hz.
    pub fn new(clock_frequency: u64) -> Self {
        Self {
            memory: [0; _],
            stack: [0; _],
            v_registers: [0; _],
            memory_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            display: [[Pixel::Black; _]; _],
            speed: Duration::from_nanos(1_000_000_000 / clock_frequency),
        }
    }

    /// Fetch the next instruction from memory.
    fn fetch(&mut self) -> [u8; 2] {
        debug_assert!(
            self.program_counter % 2 == 0,
            "The program counter must be even"
        );
        let instruction = [
            self.memory[self.program_counter as usize],
            self.memory[self.program_counter as usize + 1],
        ];
        self.program_counter += 2;
        self.program_counter = self.program_counter % self.memory.len() as u16;
        instruction
    }

    /// Execute the given instruction.
    fn execute(&mut self, instruction: Instruction) {
        use Instruction as I;

        match instruction {
            I::Nop => {}
        };
    }
}

impl Interpreter for Chip8Interpreter {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        let instruction = decode(self.fetch());
        self.execute(instruction);

        Some(self.display)
    }

    fn speed(&self) -> Duration {
        self.speed
    }

    fn buzzer_active(&self) -> bool {
        self.sound_timer > 0
    }
}
