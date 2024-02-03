//! This crate provides the instructions and the optional capabilities to encode and decode them.

#[cfg(feature = "decode")]
mod decoding;

#[cfg(feature = "decode")]
pub use self::decoding::{decode, DecodingError};

#[cfg(feature = "encode")]
mod encoding;

#[cfg(feature = "encode")]
pub use self::encoding::encode;

/// The set of instructions that are supported by the interpreter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    /// Clear the display.
    ClearScreen,

    /// Return from the current subroutine.
    Return,

    /// Jump to the given address.
    Jump(u16),

    /// Call the subroutine at the given address.
    Call(u16),

    /// Skip the next instruction if the two operands are equal. The first is a general purpose
    /// register.
    SkipIfEqual(u8, Operand),

    /// Skip the next instruction if the two operands are not equal. The first is a general purpose
    /// register.
    SkipIfNotEqual(u8, Operand),

    /// Load the value of the right operand INTO the register given by the left number.
    LoadRegister(u8, Operand),

    /// Add the right literal byte to the general purpose register of the left number, without a
    /// carry bit, and store the result in the left register.
    AddNoCarry(u8, u8),

    /// Bitwise OR the two registers together and store the result in the left.
    Or(u8, u8),

    /// Bitwise AND the two registers together and store the result in the left.
    And(u8, u8),

    /// Bitwise XOR the two registers together and store the result in the left.
    Xor(u8, u8),

    /// Add the two registers, storing the result in the left one and storing the carry bit in VF.
    AddWithCarry(u8, u8),

    /// Set Vx = Vx - Vy, and set VF to 1 if Vx > Vy, otherwise 0.
    Sub(u8, u8),

    /// Shift this register to the right by 1 place, overflowing into VF.
    ShiftRight(u8),

    /// Set Vx = Vy - Vx, and set VF to 1 if Vy > Vx, otherwise 0.
    SubN(u8, u8),

    /// Shift this register to the left by 1 place, overflowing into VF.
    ShiftLeft(u8),

    /// Load the given address into the memory register.
    LoadMemoryRegister(u16),

    /// Add V0 to the given address, and jump to that address.
    JumpPlusV0(u16),

    /// Generate a random byte, AND it with the second operand, and store it in the general purpose
    /// register given by the left.
    LoadRandomWithMask(u8, u8),

    /// Display n-byte sprite starting at the memory location in the memory register at coordinates
    /// (Vx, Vy), set VF = collision.
    ///
    /// The interpreter reads n bytes from memory, starting at the address stored in the memory
    /// register. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    /// Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is
    /// set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside
    /// the coordinates of the display, it wraps around to the opposite side of the screen.
    Draw(u8, u8, u8),

    /// Skip the next instruction if the key with the number in Vx is currently being pressed.
    SkipIfKeyPressed(u8),

    /// Skip the next instruction if the key with the number in Vx is not currently being pressed.
    SkipIfKeyNotPressed(u8),

    /// Set Vx to the value in the delay timer.
    LoadFromDelayTimer(u8),

    /// Wait for a key press, and store its number in Vx.
    WaitForKeyPress(u8),

    /// Set the delay timer to the value in Vx.
    LoadIntoDelayTimer(u8),

    /// Set the sound timer to the value in Vx.
    LoadIntoSoundTimer(u8),

    /// Add the value of Vx to the memory register, storing the result in the memory register.
    AddToMemoryRegister(u8),

    /// Load the memory register with the address of the sprite representing the bottom nibble in Vx.
    LoadDigitAddress(u8),

    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    ///
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at
    /// location in I, the tens digit at location I+1, and the ones digit at location I+2, where I
    /// is the memory register.
    StoreBcdInMemory(u8),

    /// Store registers V0 through Vx in memory starting at the location in the memory register.
    StoreRegistersInMemory(u8),

    /// Read registers V0 through Vx from memory starting at the location in the memory register.
    ReadRegistersFromMemory(u8),
}

/// An operand that can be used in an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operand {
    /// A general purpose register.
    Register(u8),

    /// A literal byte value.
    Literal(u8),
}
