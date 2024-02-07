//! This module handles encoding instructions to bytecode.

use crate::{Instruction, Operand};
use thiserror::Error;

/// A potential error when encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum EncodingError {
    /// CHIP-8 has 12-bit addresses. This error means the given address was too big.
    #[error("This address is more than 12 bits: 0x{0:0>4X}")]
    AddressTooBig(u16),

    /// There are only 16 registers in CHIP-8. This error means the given register number was too
    /// big.
    #[error("This address is more than 8 bits: 0x{0:0>2X}")]
    RegisterTooBig(u8),

    /// A nibble is only four bits, but the smallest integer type in Rust is 8 bits. This error
    /// means a `u8` that was expected to be a nibble was too big. This error is only produced when
    /// trying to encode [`Instruction::Draw`].
    #[error("This number should be one nibble: 0x{0:0>2X}")]
    NibbleTooBig(u8),
}

/// Return an error if the address is too big.
#[inline]
fn assert_addr(addr: u16) -> Result<(), EncodingError> {
    if addr & 0xF000 == 0 {
        Ok(())
    } else {
        Err(EncodingError::AddressTooBig(addr))
    }
}

/// Return an error if the register number is too big.
#[inline]
fn assert_reg(reg: u8) -> Result<(), EncodingError> {
    if reg > 15 {
        Err(EncodingError::RegisterTooBig(reg))
    } else {
        Ok(())
    }
}

/// Encode an instruction into a pair of bytes.
pub fn encode(instruction: Instruction) -> Result<[u8; 2], EncodingError> {
    use Instruction as I;
    use Operand::{Literal as Lit, Register as Reg};

    Ok(u16::to_be_bytes(match instruction {
        I::Nop => 0x0000,
        I::ClearScreen => 0x00E0,
        I::Return => 0x00EE,
        I::Jump(address) => {
            // 1nnn
            assert_addr(address)?;
            0x1000 | address
        }
        I::Call(address) => {
            // 2nnn
            assert_addr(address)?;
            0x2000 | address
        }
        I::SkipIfEqual(r1, Reg(r2)) => {
            // 5xy0
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x5000 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::SkipIfEqual(r1, Lit(byte)) => {
            // 3xkk
            assert_reg(r1)?;
            0x3000 | (r1 as u16) << 8 | byte as u16
        }
        I::SkipIfNotEqual(r1, Reg(r2)) => {
            // 9xy0
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x9000 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::SkipIfNotEqual(r1, Lit(byte)) => {
            // 4xkk
            assert_reg(r1)?;
            0x4000 | (r1 as u16) << 8 | byte as u16
        }
        I::LoadRegister(r1, Reg(r2)) => {
            // 8xy0
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x8000 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::LoadRegister(r1, Lit(byte)) => {
            // 6xkk
            assert_reg(r1)?;
            0x6000 | (r1 as u16) << 8 | byte as u16
        }
        I::AddNoCarry(r1, byte) => {
            // 7xkk
            assert_reg(r1)?;
            0x7000 | (r1 as u16) << 8 | byte as u16
        }
        I::Or(r1, r2) => {
            // 8xy1
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x8001 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::And(r1, r2) => {
            // 8xy2
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x8002 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::Xor(r1, r2) => {
            // 8xy3
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x8003 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::AddWithCarry(r1, r2) => {
            // 8xy4
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x8004 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::Sub(r1, r2) => {
            // 8xy5
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x8005 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::ShiftRight(reg) => {
            // 8x_6
            assert_reg(reg)?;
            0x8006 | (reg as u16) << 8
        }
        I::SubN(r1, r2) => {
            // 8xy7
            assert_reg(r1)?;
            assert_reg(r2)?;
            0x8007 | (r1 as u16) << 8 | (r2 as u16) << 4
        }
        I::ShiftLeft(reg) => {
            // 8x_E
            assert_reg(reg)?;
            0x800E | (reg as u16) << 8
        }
        I::LoadMemoryRegister(address) => {
            // Annn
            assert_addr(address)?;
            0xA000 | address
        }
        I::JumpPlusV0(address) => {
            // Bnnn
            assert_addr(address)?;
            0xB000 | address
        }
        I::LoadRandomWithMask(reg, mask) => {
            // Cxkk
            assert_reg(reg)?;
            0xC000 | (reg as u16) << 8 | mask as u16
        }
        I::Draw(x, y, n) => {
            // Dxyn
            assert_reg(x)?;
            assert_reg(y)?;
            if n > 15 {
                return Err(EncodingError::NibbleTooBig(n));
            }
            0xD000 | (x as u16) << 8 | (y as u16) << 4 | n as u16
        }
        I::SkipIfKeyPressed(reg) => {
            // Ex9E
            assert_reg(reg)?;
            0xE09E | (reg as u16) << 8
        }
        I::SkipIfKeyNotPressed(reg) => {
            // ExA1
            assert_reg(reg)?;
            0xE0A1 | (reg as u16) << 8
        }
        I::LoadFromDelayTimer(reg) => {
            // Fx07
            assert_reg(reg)?;
            0xF007 | (reg as u16) << 8
        }
        I::WaitForKeyPress(reg) => {
            // Fx0A
            assert_reg(reg)?;
            0xF00A | (reg as u16) << 8
        }
        I::LoadIntoDelayTimer(reg) => {
            // Fx15
            assert_reg(reg)?;
            0xF015 | (reg as u16) << 8
        }
        I::LoadIntoSoundTimer(reg) => {
            // Fx18
            assert_reg(reg)?;
            0xF018 | (reg as u16) << 8
        }
        I::AddToMemoryRegister(reg) => {
            // Fx1E
            assert_reg(reg)?;
            0xF01E | (reg as u16) << 8
        }
        I::LoadDigitAddress(reg) => {
            // Fx29
            assert_reg(reg)?;
            0xF029 | (reg as u16) << 8
        }
        I::StoreBcdInMemory(reg) => {
            // Fx33
            assert_reg(reg)?;
            0xF033 | (reg as u16) << 8
        }
        I::StoreRegistersInMemory(reg) => {
            // Fx55
            assert_reg(reg)?;
            0xF055 | (reg as u16) << 8
        }
        I::ReadRegistersFromMemory(reg) => {
            // Fx65
            assert_reg(reg)?;
            0xF065 | (reg as u16) << 8
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_success_test() {
        use Instruction as I;
        use Operand::{Literal as Lit, Register as Reg};

        fn enc(instr: I) -> Result<u16, EncodingError> {
            encode(instr).map(u16::from_be_bytes)
        }

        assert_eq!(enc(I::Nop), Ok(0x0000));

        assert_eq!(enc(I::ClearScreen), Ok(0x00E0));

        assert_eq!(enc(I::Return), Ok(0x00EE));

        assert_eq!(enc(I::Jump(0x37C)), Ok(0x137C));
        assert_eq!(enc(I::Jump(0x590)), Ok(0x1590));
        assert_eq!(enc(I::Jump(0x000)), Ok(0x1000));
        assert_eq!(enc(I::Jump(0x210)), Ok(0x1210));

        assert_eq!(enc(I::Call(0x37C)), Ok(0x237C));
        assert_eq!(enc(I::Call(0x590)), Ok(0x2590));
        assert_eq!(enc(I::Call(0x000)), Ok(0x2000));
        assert_eq!(enc(I::Call(0x210)), Ok(0x2210));

        assert_eq!(enc(I::SkipIfEqual(0, Lit(0x4F))), Ok(0x304F));
        assert_eq!(enc(I::SkipIfEqual(1, Lit(0))), Ok(0x3100));
        assert_eq!(enc(I::SkipIfEqual(6, Lit(0xC8))), Ok(0x36C8));
        assert_eq!(enc(I::SkipIfEqual(13, Lit(18))), Ok(0x3D12));

        assert_eq!(enc(I::SkipIfNotEqual(0, Lit(0x4F))), Ok(0x404F));
        assert_eq!(enc(I::SkipIfNotEqual(1, Lit(0))), Ok(0x4100));
        assert_eq!(enc(I::SkipIfNotEqual(6, Lit(0xC8))), Ok(0x46C8));
        assert_eq!(enc(I::SkipIfNotEqual(13, Lit(18))), Ok(0x4D12));

        assert_eq!(enc(I::SkipIfEqual(0, Reg(4))), Ok(0x5040));
        assert_eq!(enc(I::SkipIfEqual(1, Reg(0))), Ok(0x5100));
        assert_eq!(enc(I::SkipIfEqual(6, Reg(12))), Ok(0x56C0));
        assert_eq!(enc(I::SkipIfEqual(13, Reg(1))), Ok(0x5D10));

        assert_eq!(enc(I::SkipIfNotEqual(0, Reg(4))), Ok(0x9040));
        assert_eq!(enc(I::SkipIfNotEqual(1, Reg(0))), Ok(0x9100));
        assert_eq!(enc(I::SkipIfNotEqual(6, Reg(12))), Ok(0x96C0));
        assert_eq!(enc(I::SkipIfNotEqual(13, Reg(1))), Ok(0x9D10));

        assert_eq!(enc(I::LoadRegister(1, Lit(0xFC))), Ok(0x61FC));
        assert_eq!(enc(I::LoadRegister(4, Lit(1))), Ok(0x6401));
        assert_eq!(enc(I::LoadRegister(9, Lit(0xFF))), Ok(0x69FF));
        assert_eq!(enc(I::LoadRegister(14, Lit(14))), Ok(0x6E0E));

        assert_eq!(enc(I::AddNoCarry(2, 0x8D)), Ok(0x728D));
        assert_eq!(enc(I::AddNoCarry(10, 0x56)), Ok(0x7A56));
        assert_eq!(enc(I::AddNoCarry(4, 15)), Ok(0x740F));
        assert_eq!(enc(I::AddNoCarry(0, 0x19)), Ok(0x7019));

        assert_eq!(enc(I::LoadRegister(0, Reg(1))), Ok(0x8010));
        assert_eq!(enc(I::LoadRegister(4, Reg(9))), Ok(0x8490));
        assert_eq!(enc(I::LoadRegister(8, Reg(0))), Ok(0x8800));
        assert_eq!(enc(I::LoadRegister(12, Reg(10))), Ok(0x8CA0));
        assert_eq!(enc(I::LoadRegister(15, Reg(2))), Ok(0x8F20));
        assert_eq!(enc(I::LoadRegister(3, Reg(12))), Ok(0x83C0));

        assert_eq!(enc(I::Or(0, 1)), Ok(0x8011));
        assert_eq!(enc(I::Or(4, 9)), Ok(0x8491));
        assert_eq!(enc(I::Or(8, 0)), Ok(0x8801));
        assert_eq!(enc(I::Or(12, 10)), Ok(0x8CA1));
        assert_eq!(enc(I::Or(15, 2)), Ok(0x8F21));
        assert_eq!(enc(I::Or(3, 12)), Ok(0x83C1));

        assert_eq!(enc(I::And(0, 1)), Ok(0x8012));
        assert_eq!(enc(I::And(4, 9)), Ok(0x8492));
        assert_eq!(enc(I::And(8, 0)), Ok(0x8802));
        assert_eq!(enc(I::And(12, 10)), Ok(0x8CA2));
        assert_eq!(enc(I::And(15, 2)), Ok(0x8F22));
        assert_eq!(enc(I::And(3, 12)), Ok(0x83C2));

        assert_eq!(enc(I::Xor(0, 1)), Ok(0x8013));
        assert_eq!(enc(I::Xor(4, 9)), Ok(0x8493));
        assert_eq!(enc(I::Xor(8, 0)), Ok(0x8803));
        assert_eq!(enc(I::Xor(12, 10)), Ok(0x8CA3));
        assert_eq!(enc(I::Xor(15, 2)), Ok(0x8F23));
        assert_eq!(enc(I::Xor(3, 12)), Ok(0x83C3));

        assert_eq!(enc(I::AddWithCarry(0, 1)), Ok(0x8014));
        assert_eq!(enc(I::AddWithCarry(4, 9)), Ok(0x8494));
        assert_eq!(enc(I::AddWithCarry(8, 0)), Ok(0x8804));
        assert_eq!(enc(I::AddWithCarry(12, 10)), Ok(0x8CA4));
        assert_eq!(enc(I::AddWithCarry(15, 2)), Ok(0x8F24));
        assert_eq!(enc(I::AddWithCarry(3, 12)), Ok(0x83C4));

        assert_eq!(enc(I::Sub(0, 1)), Ok(0x8015));
        assert_eq!(enc(I::Sub(4, 9)), Ok(0x8495));
        assert_eq!(enc(I::Sub(8, 0)), Ok(0x8805));
        assert_eq!(enc(I::Sub(12, 10)), Ok(0x8CA5));
        assert_eq!(enc(I::Sub(15, 2)), Ok(0x8F25));
        assert_eq!(enc(I::Sub(3, 12)), Ok(0x83C5));

        assert_eq!(enc(I::ShiftRight(0)), Ok(0x8006));
        assert_eq!(enc(I::ShiftRight(4)), Ok(0x8406));
        assert_eq!(enc(I::ShiftRight(8)), Ok(0x8806));
        assert_eq!(enc(I::ShiftRight(12)), Ok(0x8C06));
        assert_eq!(enc(I::ShiftRight(15)), Ok(0x8F06));
        assert_eq!(enc(I::ShiftRight(3)), Ok(0x8306));

        assert_eq!(enc(I::SubN(0, 1)), Ok(0x8017));
        assert_eq!(enc(I::SubN(4, 9)), Ok(0x8497));
        assert_eq!(enc(I::SubN(8, 0)), Ok(0x8807));
        assert_eq!(enc(I::SubN(12, 10)), Ok(0x8CA7));
        assert_eq!(enc(I::SubN(15, 2)), Ok(0x8F27));
        assert_eq!(enc(I::SubN(3, 12)), Ok(0x83C7));

        assert_eq!(enc(I::ShiftLeft(0)), Ok(0x800E));
        assert_eq!(enc(I::ShiftLeft(4)), Ok(0x840E));
        assert_eq!(enc(I::ShiftLeft(8)), Ok(0x880E));
        assert_eq!(enc(I::ShiftLeft(12)), Ok(0x8C0E));
        assert_eq!(enc(I::ShiftLeft(15)), Ok(0x8F0E));
        assert_eq!(enc(I::ShiftLeft(3)), Ok(0x830E));

        assert_eq!(enc(I::LoadMemoryRegister(0x375)), Ok(0xA375));
        assert_eq!(enc(I::LoadMemoryRegister(0x200)), Ok(0xA200));
        assert_eq!(enc(I::LoadMemoryRegister(0x9FD)), Ok(0xA9FD));
        assert_eq!(enc(I::LoadMemoryRegister(0xA42)), Ok(0xAA42));

        assert_eq!(enc(I::JumpPlusV0(0x375)), Ok(0xB375));
        assert_eq!(enc(I::JumpPlusV0(0x200)), Ok(0xB200));
        assert_eq!(enc(I::JumpPlusV0(0x9FD)), Ok(0xB9FD));
        assert_eq!(enc(I::JumpPlusV0(0xA42)), Ok(0xBA42));

        assert_eq!(enc(I::LoadRandomWithMask(2, 0x34)), Ok(0xC234));
        assert_eq!(enc(I::LoadRandomWithMask(0, 0x00)), Ok(0xC000));
        assert_eq!(enc(I::LoadRandomWithMask(4, 0xFF)), Ok(0xC4FF));
        assert_eq!(enc(I::LoadRandomWithMask(14, 0xAA)), Ok(0xCEAA));

        assert_eq!(enc(I::Draw(0, 1, 5)), Ok(0xD015));
        assert_eq!(enc(I::Draw(4, 0, 9)), Ok(0xD409));
        assert_eq!(enc(I::Draw(7, 8, 2)), Ok(0xD782));
        assert_eq!(enc(I::Draw(4, 7, 13)), Ok(0xD47D));
        assert_eq!(enc(I::Draw(6, 6, 15)), Ok(0xD66F));
        assert_eq!(enc(I::Draw(14, 4, 10)), Ok(0xDE4A));

        assert_eq!(enc(I::SkipIfKeyPressed(0)), Ok(0xE09E));
        assert_eq!(enc(I::SkipIfKeyPressed(4)), Ok(0xE49E));
        assert_eq!(enc(I::SkipIfKeyPressed(9)), Ok(0xE99E));
        assert_eq!(enc(I::SkipIfKeyPressed(11)), Ok(0xEB9E));

        assert_eq!(enc(I::SkipIfKeyNotPressed(0)), Ok(0xE0A1));
        assert_eq!(enc(I::SkipIfKeyNotPressed(4)), Ok(0xE4A1));
        assert_eq!(enc(I::SkipIfKeyNotPressed(9)), Ok(0xE9A1));
        assert_eq!(enc(I::SkipIfKeyNotPressed(11)), Ok(0xEBA1));

        assert_eq!(enc(I::LoadFromDelayTimer(1)), Ok(0xF107));
        assert_eq!(enc(I::LoadFromDelayTimer(3)), Ok(0xF307));
        assert_eq!(enc(I::LoadFromDelayTimer(6)), Ok(0xF607));
        assert_eq!(enc(I::LoadFromDelayTimer(8)), Ok(0xF807));
        assert_eq!(enc(I::LoadFromDelayTimer(12)), Ok(0xFC07));
        assert_eq!(enc(I::LoadFromDelayTimer(14)), Ok(0xFE07));

        assert_eq!(enc(I::WaitForKeyPress(1)), Ok(0xF10A));
        assert_eq!(enc(I::WaitForKeyPress(3)), Ok(0xF30A));
        assert_eq!(enc(I::WaitForKeyPress(6)), Ok(0xF60A));
        assert_eq!(enc(I::WaitForKeyPress(8)), Ok(0xF80A));
        assert_eq!(enc(I::WaitForKeyPress(12)), Ok(0xFC0A));
        assert_eq!(enc(I::WaitForKeyPress(14)), Ok(0xFE0A));

        assert_eq!(enc(I::LoadIntoDelayTimer(1)), Ok(0xF115));
        assert_eq!(enc(I::LoadIntoDelayTimer(3)), Ok(0xF315));
        assert_eq!(enc(I::LoadIntoDelayTimer(6)), Ok(0xF615));
        assert_eq!(enc(I::LoadIntoDelayTimer(8)), Ok(0xF815));
        assert_eq!(enc(I::LoadIntoDelayTimer(12)), Ok(0xFC15));
        assert_eq!(enc(I::LoadIntoDelayTimer(14)), Ok(0xFE15));

        assert_eq!(enc(I::LoadIntoSoundTimer(1)), Ok(0xF118));
        assert_eq!(enc(I::LoadIntoSoundTimer(3)), Ok(0xF318));
        assert_eq!(enc(I::LoadIntoSoundTimer(6)), Ok(0xF618));
        assert_eq!(enc(I::LoadIntoSoundTimer(8)), Ok(0xF818));
        assert_eq!(enc(I::LoadIntoSoundTimer(12)), Ok(0xFC18));
        assert_eq!(enc(I::LoadIntoSoundTimer(14)), Ok(0xFE18));

        assert_eq!(enc(I::AddToMemoryRegister(1)), Ok(0xF11E));
        assert_eq!(enc(I::AddToMemoryRegister(3)), Ok(0xF31E));
        assert_eq!(enc(I::AddToMemoryRegister(6)), Ok(0xF61E));
        assert_eq!(enc(I::AddToMemoryRegister(8)), Ok(0xF81E));
        assert_eq!(enc(I::AddToMemoryRegister(12)), Ok(0xFC1E));
        assert_eq!(enc(I::AddToMemoryRegister(14)), Ok(0xFE1E));

        assert_eq!(enc(I::LoadDigitAddress(1)), Ok(0xF129));
        assert_eq!(enc(I::LoadDigitAddress(3)), Ok(0xF329));
        assert_eq!(enc(I::LoadDigitAddress(6)), Ok(0xF629));
        assert_eq!(enc(I::LoadDigitAddress(8)), Ok(0xF829));
        assert_eq!(enc(I::LoadDigitAddress(12)), Ok(0xFC29));
        assert_eq!(enc(I::LoadDigitAddress(14)), Ok(0xFE29));

        assert_eq!(enc(I::StoreBcdInMemory(1)), Ok(0xF133));
        assert_eq!(enc(I::StoreBcdInMemory(3)), Ok(0xF333));
        assert_eq!(enc(I::StoreBcdInMemory(6)), Ok(0xF633));
        assert_eq!(enc(I::StoreBcdInMemory(8)), Ok(0xF833));
        assert_eq!(enc(I::StoreBcdInMemory(12)), Ok(0xFC33));
        assert_eq!(enc(I::StoreBcdInMemory(14)), Ok(0xFE33));

        assert_eq!(enc(I::StoreRegistersInMemory(1)), Ok(0xF155));
        assert_eq!(enc(I::StoreRegistersInMemory(3)), Ok(0xF355));
        assert_eq!(enc(I::StoreRegistersInMemory(6)), Ok(0xF655));
        assert_eq!(enc(I::StoreRegistersInMemory(8)), Ok(0xF855));
        assert_eq!(enc(I::StoreRegistersInMemory(12)), Ok(0xFC55));
        assert_eq!(enc(I::StoreRegistersInMemory(14)), Ok(0xFE55));

        assert_eq!(enc(I::ReadRegistersFromMemory(1)), Ok(0xF165));
        assert_eq!(enc(I::ReadRegistersFromMemory(3)), Ok(0xF365));
        assert_eq!(enc(I::ReadRegistersFromMemory(6)), Ok(0xF665));
        assert_eq!(enc(I::ReadRegistersFromMemory(8)), Ok(0xF865));
        assert_eq!(enc(I::ReadRegistersFromMemory(12)), Ok(0xFC65));
        assert_eq!(enc(I::ReadRegistersFromMemory(14)), Ok(0xFE65));
    }

    #[test]
    fn encode_error_test() {
        use EncodingError as E;
        use Instruction as I;

        assert_eq!(encode(I::Jump(0x1000)), Err(E::AddressTooBig(0x1000)));
        assert_eq!(encode(I::Jump(0x1234)), Err(E::AddressTooBig(0x1234)));
        assert_eq!(encode(I::Jump(0xFFFF)), Err(E::AddressTooBig(0xFFFF)));
        assert_eq!(encode(I::Jump(0x4F9A)), Err(E::AddressTooBig(0x4F9A)));

        assert_eq!(encode(I::Call(0x1000)), Err(E::AddressTooBig(0x1000)));
        assert_eq!(encode(I::Call(0x1234)), Err(E::AddressTooBig(0x1234)));
        assert_eq!(encode(I::Call(0xFFFF)), Err(E::AddressTooBig(0xFFFF)));
        assert_eq!(encode(I::Call(0x4F9A)), Err(E::AddressTooBig(0x4F9A)));

        assert_eq!(encode(I::AddNoCarry(16, 0x56)), Err(E::RegisterTooBig(16)));
        assert_eq!(encode(I::AddNoCarry(26, 0x01)), Err(E::RegisterTooBig(26)));
        assert_eq!(
            encode(I::AddNoCarry(255, 0x50)),
            Err(E::RegisterTooBig(255))
        );
        assert_eq!(
            encode(I::AddNoCarry(102, 0xC0)),
            Err(E::RegisterTooBig(102))
        );

        assert_eq!(encode(I::AddWithCarry(16, 8)), Err(E::RegisterTooBig(16)));
        assert_eq!(encode(I::AddWithCarry(34, 3)), Err(E::RegisterTooBig(34)));
        assert_eq!(
            encode(I::AddWithCarry(178, 150)),
            Err(E::RegisterTooBig(178))
        );
        assert_eq!(encode(I::AddWithCarry(8, 16)), Err(E::RegisterTooBig(16)));
        assert_eq!(encode(I::AddWithCarry(3, 34)), Err(E::RegisterTooBig(34)));
        assert_eq!(
            encode(I::AddWithCarry(150, 178)),
            Err(E::RegisterTooBig(150))
        );

        assert_eq!(encode(I::Draw(1, 2, 16)), Err(E::NibbleTooBig(16)));
        assert_eq!(encode(I::Draw(9, 3, 87)), Err(E::NibbleTooBig(87)));
        assert_eq!(encode(I::Draw(13, 0, 200)), Err(E::NibbleTooBig(200)));
        assert_eq!(encode(I::Draw(10, 4, 186)), Err(E::NibbleTooBig(186)));
        assert_eq!(encode(I::Draw(100, 4, 186)), Err(E::RegisterTooBig(100)));
        assert_eq!(encode(I::Draw(10, 40, 186)), Err(E::RegisterTooBig(40)));
    }
}
