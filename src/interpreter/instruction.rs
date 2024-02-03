//! This module provides the instructions and the capability to decode them.

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

/// A potential error when decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodingError {
    /// The bytecode was not recognised as a valid instruction.
    UnrecognisedBytecode(u16),
}

/// Decode a pair of bytes into an instruction, panicking if the decoding fails.
///
/// See <http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.0> for a list of all instructions.
pub fn decode(bytes: [u8; 2]) -> Result<Instruction, DecodingError> {
    use Instruction as I;
    use Operand::{Literal as Lit, Register as Reg};

    let [b1, b2] = bytes;

    // Nibbles
    let n1 = (b1 & 0xF0) >> 4;
    let n2 = b1 & 0x0F;
    let n3 = (b2 & 0xF0) >> 4;
    let n4 = b2 & 0x0F;

    Ok(match (n1, n2, n3, n4) {
        (0, 0, 0xE, 0) => I::ClearScreen,
        (0, 0, 0xE, 0xE) => I::Return,
        (1, n2, n3, n4) => {
            let address = ((n2 as u16) << 8) + ((n3 as u16) << 4) + n4 as u16;
            debug_assert!(
                address & 0xF000 == 0,
                "Addresses should only ever be 12 bits"
            );
            I::Jump(address)
        }
        (2, n2, n3, n4) => {
            let address = ((n2 as u16) << 8) + ((n3 as u16) << 4) + n4 as u16;
            debug_assert!(
                address & 0xF000 == 0,
                "Addresses should only ever be 12 bits"
            );
            I::Call(address)
        }
        (3, x, _, _) => I::SkipIfEqual(x, Lit(b2)),
        (4, x, _, _) => I::SkipIfNotEqual(x, Lit(b2)),
        (5, x, y, 0) => I::SkipIfEqual(x, Reg(y)),
        (6, x, _, _) => I::LoadRegister(x, Lit(b2)),
        (7, x, _, _) => I::AddNoCarry(x, b2),
        (8, x, y, 0) => I::LoadRegister(x, Reg(y)),
        (8, x, y, 1) => I::Or(x, y),
        (8, x, y, 2) => I::And(x, y),
        (8, x, y, 3) => I::Xor(x, y),
        (8, x, y, 4) => I::AddWithCarry(x, y),
        (8, x, y, 5) => I::Sub(x, y),
        (8, x, _, 6) => I::ShiftRight(x),
        (8, x, y, 7) => I::SubN(x, y),
        (8, x, _, 0xE) => I::ShiftLeft(x),
        (9, x, y, 0) => I::SkipIfNotEqual(x, Reg(y)),
        (0xA, n2, n3, n4) => {
            let address = ((n2 as u16) << 8) + ((n3 as u16) << 4) + n4 as u16;
            debug_assert!(
                address & 0xF000 == 0,
                "Addresses should only ever be 12 bits"
            );
            I::LoadMemoryRegister(address)
        }
        (0xB, n2, n3, n4) => {
            let address = ((n2 as u16) << 8) + ((n3 as u16) << 4) + n4 as u16;
            debug_assert!(
                address & 0xF000 == 0,
                "Addresses should only ever be 12 bits"
            );
            I::JumpPlusV0(address)
        }
        (0xC, x, _, _) => I::LoadRandomWithMask(x, b2),
        (0xD, x, y, n) => I::Draw(x, y, n),
        (0xE, x, 9, 0xE) => I::SkipIfKeyPressed(x),
        (0xE, x, 0xA, 1) => I::SkipIfKeyNotPressed(x),
        (0xF, x, 0, 7) => I::LoadFromDelayTimer(x),
        (0xF, x, 0, 0xA) => I::WaitForKeyPress(x),
        (0xF, x, 1, 5) => I::LoadIntoDelayTimer(x),
        (0xF, x, 1, 8) => I::LoadIntoSoundTimer(x),
        (0xF, x, 1, 0xE) => I::AddToMemoryRegister(x),
        (0xF, x, 2, 9) => I::LoadDigitAddress(x),
        (0xF, x, 3, 3) => I::StoreBcdInMemory(x),
        (0xF, x, 5, 5) => I::StoreRegistersInMemory(x),
        (0xF, x, 6, 5) => I::ReadRegistersFromMemory(x),
        _ => {
            return Err(DecodingError::UnrecognisedBytecode(u16::from_be_bytes([
                b1, b2,
            ])))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_test() {
        use Instruction as I;

        assert_eq!(decode([0x00, 0xE0]), Ok(I::ClearScreen));

        assert_eq!(decode([0x00, 0xEE]), Ok(I::Return));

        assert_eq!(decode([0x13, 0x7C]), Ok(I::Jump(0x37C)));
        assert_eq!(decode([0x15, 0x90]), Ok(I::Jump(0x590)));
        assert_eq!(decode([0x10, 0x00]), Ok(I::Jump(0x000)));
        assert_eq!(decode([0x12, 0x10]), Ok(I::Jump(0x210)));

        assert_eq!(decode([0x23, 0x7C]), Ok(I::Call(0x37C)));
        assert_eq!(decode([0x25, 0x90]), Ok(I::Call(0x590)));
        assert_eq!(decode([0x20, 0x00]), Ok(I::Call(0x000)));
        assert_eq!(decode([0x22, 0x10]), Ok(I::Call(0x210)));
    }
}
