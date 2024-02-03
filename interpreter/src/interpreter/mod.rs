//! This module contains the [`Interpreter`] type.

mod memory;

use self::memory::init_memory;
use crate::interpreter::memory::FONT_ADDRESS_START;
use chip8_base::{Display, Interpreter, Keys, Pixel};
use chip8_instructions::{decode, DecodingError, Instruction, Operand};
use std::time::{Duration, Instant};

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

    /// The time when we last decremented the timers.
    last_timer_decrement: Instant,

    /// Are we currently waiting for a key to be pressed? If so, which register should it go into?
    waiting_for_key_press: Option<u8>,
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
            last_timer_decrement: Instant::now(),
            waiting_for_key_press: None,
        }
    }

    /// Load the value from the given register.
    #[inline]
    fn reg(&self, x: u8) -> u8 {
        debug_assert!(x & 0xF0 == 0, "Register numbers should only be 4 bits");
        self.v_registers[x as usize]
    }

    #[inline]
    fn mut_reg(&mut self, x: u8) -> &mut u8 {
        debug_assert!(x & 0xF0 == 0, "Register numbers should only be 4 bits");
        &mut self.v_registers[x as usize]
    }

    /// Get the value of the operand.
    #[inline]
    fn get_operand(&self, op: Operand) -> u8 {
        match op {
            Operand::Register(x) => self.reg(x),
            Operand::Literal(byte) => byte,
        }
    }

    /// Fetch the next instruction from memory.
    fn fetch(&mut self) -> [u8; 2] {
        //debug_assert!(
        //self.program_counter % 2 == 0,
        //"The program counter must be even"
        //);
        let instruction = [
            self.memory[self.program_counter as usize],
            self.memory[self.program_counter as usize + 1],
        ];
        self.program_counter += 2;
        self.program_counter = self.program_counter % self.memory.len() as u16;
        instruction
    }

    /// Execute the given instruction.
    fn execute(&mut self, instruction: Instruction, keys: &Keys) {
        use Instruction as I;

        match instruction {
            I::ClearScreen => self.display = [[Pixel::Black; _]; _],
            I::Return => {
                self.stack_pointer = self
                    .stack_pointer
                    .checked_sub(1)
                    .expect("The stack pointer should never go negative");
                self.program_counter = self.stack[self.stack_pointer as usize];
            }
            I::Jump(address) => self.program_counter = address,
            I::Call(address) => {
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = address;
            }
            I::SkipIfEqual(x, op) => {
                if self.reg(x) == self.get_operand(op) {
                    self.program_counter += 2;
                }
            }
            I::SkipIfNotEqual(x, op) => {
                if self.reg(x) != self.get_operand(op) {
                    self.program_counter += 2;
                }
            }
            I::LoadRegister(x, op) => *self.mut_reg(x) = self.get_operand(op),
            I::AddNoCarry(x, byte) => *self.mut_reg(x) = self.reg(x).wrapping_add(byte),
            I::Or(x, y) => *self.mut_reg(x) |= self.reg(y),
            I::And(x, y) => *self.mut_reg(x) &= self.reg(y),
            I::Xor(x, y) => *self.mut_reg(x) ^= self.reg(y),
            I::AddWithCarry(x, y) => {
                let (value, carry) = self.reg(x).overflowing_add(self.reg(y));
                *self.mut_reg(x) = value;
                *self.mut_reg(0xF) = carry as u8;
            }
            I::Sub(x, y) => {
                let (value, carry) = self.reg(x).overflowing_sub(self.reg(y));
                *self.mut_reg(x) = value;
                *self.mut_reg(0xF) = carry as u8;
            }
            I::ShiftRight(x) => {
                *self.mut_reg(0xF) = self.reg(x) & 1;
                *self.mut_reg(x) = self.reg(x) >> 1;
            }
            I::SubN(x, y) => {
                let (value, carry) = self.reg(y).overflowing_sub(self.reg(x));
                *self.mut_reg(x) = value;
                *self.mut_reg(0xF) = carry as u8;
            }
            I::ShiftLeft(x) => {
                *self.mut_reg(0xF) = self.reg(x) & 0b1000_0000;
                *self.mut_reg(x) = self.reg(x) << 1;
            }
            I::LoadMemoryRegister(address) => self.memory_register = address,
            I::JumpPlusV0(address) => {
                let address = (address + self.reg(0) as u16) & 0xFFF;
                self.program_counter = address;
            }
            I::LoadRandomWithMask(x, mask) => *self.mut_reg(x) = rand::random::<u8>() & mask,
            I::Draw(x, y, n) => {
                let first_x = (self.reg(x) % 64) as usize;
                let mut x = first_x;
                let mut y = (self.reg(y) % 32) as usize;
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
                        //if old_pixel ^ pixel != old_pixel {
                        if (old_pixel & pixel).into() {
                            self.v_registers[0xF] = 1;
                        }
                        x += 1;
                    }
                    x = first_x;
                    y += 1;
                }
            }
            I::SkipIfKeyPressed(x) => {
                if keys[self.reg(x) as usize] {
                    self.program_counter += 2;
                }
            }
            I::SkipIfKeyNotPressed(x) => {
                if !keys[self.reg(x) as usize] {
                    self.program_counter += 2;
                }
            }
            I::LoadFromDelayTimer(x) => *self.mut_reg(x) = self.delay_timer,
            I::WaitForKeyPress(x) => self.waiting_for_key_press = Some(x),
            I::LoadIntoDelayTimer(x) => self.delay_timer = self.reg(x),
            I::LoadIntoSoundTimer(x) => self.sound_timer = self.reg(x),
            I::AddToMemoryRegister(x) => {
                self.memory_register = (self.memory_register + self.reg(x) as u16) & 0xFFF
            }
            I::LoadDigitAddress(x) => {
                self.memory_register = FONT_ADDRESS_START as u16 + 5 * (self.reg(x) & 0xF) as u16
            }
            I::StoreBcdInMemory(x) => {
                let num = self.reg(x);
                let hundreds = (num - (num % 100)) / 100;
                let tens = (num - (num % 10) - hundreds * 100) / 10;
                let units = num - hundreds * 100 - tens * 10;
                self.memory[self.memory_register as usize] = hundreds;
                self.memory[self.memory_register as usize + 1] = tens;
                self.memory[self.memory_register as usize + 2] = units;
            }
            I::StoreRegistersInMemory(reg_num) => {
                for x in 0..=reg_num {
                    self.memory[self.memory_register as usize + x as usize] = self.reg(x);
                }
            }
            I::ReadRegistersFromMemory(reg_num) => {
                for x in 0..=reg_num {
                    *self.mut_reg(x) = self.memory[self.memory_register as usize + x as usize];
                }
            }
        };
    }

    /// Decrement the timers if it's been sufficiently long since they were last decremented. The
    /// timers should be decremented at a frequency of 60 Hz.
    fn decrement_timers(&mut self) {
        if self.last_timer_decrement.elapsed() >= Duration::from_secs_f32(1. / 60.) {
            self.last_timer_decrement = Instant::now();
            self.delay_timer = self.delay_timer.saturating_sub(1);
            self.sound_timer = self.sound_timer.saturating_sub(1);
        }
    }
}

impl Interpreter for Chip8Interpreter {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        if let Some(x) = self.waiting_for_key_press {
            if let Some((key_num, _)) = keys.iter().enumerate().find(|(_idx, pressed)| **pressed) {
                *self.mut_reg(x) = key_num as u8;
                self.waiting_for_key_press = None;
            }
        } else {
            let instruction = match decode(self.fetch()) {
                Ok(instruction) => instruction,
                Err(DecodingError::UnrecognisedBytecode(bytecode)) => panic!(
                    "Unrecognised instruction with bytecode 0x{bytecode:0>4X} at address 0x{:0>4X}",
                    self.program_counter - 2
                ),
            };
            self.execute(instruction, keys);
        }

        self.decrement_timers();

        Some(self.display)
    }

    fn speed(&self) -> Duration {
        self.speed
    }

    fn buzzer_active(&self) -> bool {
        self.sound_timer > 0
    }
}
