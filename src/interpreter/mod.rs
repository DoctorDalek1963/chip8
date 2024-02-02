//! This module contains the [`Interpreter`] type.

mod instruction;
mod memory;

use self::{
    instruction::{decode, DecodingError, Instruction},
    memory::init_memory,
};
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
    pub fn new(rom: &[u8], clock_frequency: f32) -> Self {
        Self {
            memory: init_memory(rom),
            stack: [0; _],
            v_registers: [0; _],
            memory_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x200,
            stack_pointer: 0,
            display: [[Pixel::Black; _]; _],
            speed: Duration::from_secs_f32(clock_frequency.recip()),
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
    fn execute(&mut self, instruction: Instruction, _keys: &Keys) {
        use self::instruction::Operand as Op;
        use Instruction as I;

        match instruction {
            I::ClearScreen => self.display = [[Pixel::Black; _]; _],
            I::Jump(address) => self.program_counter = address,
            I::LoadRegister(x, operand) => {
                self.v_registers[x as usize] = match operand {
                    Op::Register(y) => self.v_registers[y as usize],
                    Op::Literal(byte) => byte,
                }
            }
            I::AddNoCarry(x, byte) => {
                self.v_registers[x as usize] = self.v_registers[x as usize].wrapping_add(byte)
            }
            I::LoadMemoryRegister(address) => self.memory_register = address,
            I::Draw(x, y, n) => {
                let first_x = (self.v_registers[x as usize] % 64) as usize;
                let mut x = first_x;
                let mut y = (self.v_registers[y as usize] % 32) as usize;
                self.v_registers[0xF] = 0;

                for offset in 0..n {
                    let row = self.memory[self.memory_register as usize + offset as usize];
                    if y >= 32 {
                        return;
                    }

                    for pixel in (0..=7).rev().map(|pos| {
                        if row & (1 << pos) > 0 {
                            Pixel::White
                        } else {
                            Pixel::Black
                        }
                    }) {
                        if x >= 64 {
                            break;
                        }

                        let old_pixel = self.display[y][x];
                        self.display[y][x] = old_pixel ^ pixel;

                        // Set VF if the pixel was erased
                        if old_pixel ^ pixel != old_pixel {
                            self.v_registers[0xF] = 1;
                        }
                        x += 1;
                    }
                    x = first_x;
                    y += 1;
                }
            }
            _ => unimplemented!("Instruction {instruction:?} has not been implemented to execute"),
        };
    }
}

impl Interpreter for Chip8Interpreter {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        let instruction = match decode(self.fetch()) {
            Ok(instruction) => instruction,
            Err(DecodingError::UnrecognisedBytecode(bytecode)) => panic!(
                "Unrecognised instruction with bytecode 0x{bytecode:0>4X} at address 0x{:0>4X}",
                self.program_counter - 2
            ),
        };
        self.execute(instruction, keys);

        Some(self.display)
    }

    fn speed(&self) -> Duration {
        self.speed
    }

    fn buzzer_active(&self) -> bool {
        self.sound_timer > 0
    }
}
