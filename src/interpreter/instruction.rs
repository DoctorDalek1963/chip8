//! This module provides the instructions and the capability to decode them.

/// The set of instructions that are supported by the interpreter.
pub enum Instruction {
    /// Do nothing.
    Nop
}

/// Decode a pair of bytes into an instruction, panicking if the decoding fails.
pub fn decode(bytes: [u8; 2]) -> Instruction {
    use Instruction as I;

    I::Nop
}
