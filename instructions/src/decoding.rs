//! This module handles decoding instructions from bytecode.

use crate::{Instruction, Operand};

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

    fn dec(instr: u16) -> Result<Instruction, DecodingError> {
        decode(instr.to_be_bytes())
    }

    #[test]
    fn decode_success_test() {
        use Instruction as I;
        use Operand::{Literal as Lit, Register as Reg};

        assert_eq!(dec(0x00E0), Ok(I::ClearScreen));

        assert_eq!(dec(0x00EE), Ok(I::Return));

        assert_eq!(dec(0x137C), Ok(I::Jump(0x37C)));
        assert_eq!(dec(0x1590), Ok(I::Jump(0x590)));
        assert_eq!(dec(0x1000), Ok(I::Jump(0x000)));
        assert_eq!(dec(0x1210), Ok(I::Jump(0x210)));

        assert_eq!(dec(0x237C), Ok(I::Call(0x37C)));
        assert_eq!(dec(0x2590), Ok(I::Call(0x590)));
        assert_eq!(dec(0x2000), Ok(I::Call(0x000)));
        assert_eq!(dec(0x2210), Ok(I::Call(0x210)));

        assert_eq!(dec(0x304F), Ok(I::SkipIfEqual(0, Lit(0x4F))));
        assert_eq!(dec(0x3100), Ok(I::SkipIfEqual(1, Lit(0))));
        assert_eq!(dec(0x36C8), Ok(I::SkipIfEqual(6, Lit(0xC8))));
        assert_eq!(dec(0x3D12), Ok(I::SkipIfEqual(13, Lit(18))));

        assert_eq!(dec(0x404F), Ok(I::SkipIfNotEqual(0, Lit(0x4F))));
        assert_eq!(dec(0x4100), Ok(I::SkipIfNotEqual(1, Lit(0))));
        assert_eq!(dec(0x46C8), Ok(I::SkipIfNotEqual(6, Lit(0xC8))));
        assert_eq!(dec(0x4D12), Ok(I::SkipIfNotEqual(13, Lit(18))));

        assert_eq!(dec(0x5040), Ok(I::SkipIfEqual(0, Reg(4))));
        assert_eq!(dec(0x5100), Ok(I::SkipIfEqual(1, Reg(0))));
        assert_eq!(dec(0x56C0), Ok(I::SkipIfEqual(6, Reg(12))));
        assert_eq!(dec(0x5D10), Ok(I::SkipIfEqual(13, Reg(1))));

        assert_eq!(dec(0x9040), Ok(I::SkipIfNotEqual(0, Reg(4))));
        assert_eq!(dec(0x9100), Ok(I::SkipIfNotEqual(1, Reg(0))));
        assert_eq!(dec(0x96C0), Ok(I::SkipIfNotEqual(6, Reg(12))));
        assert_eq!(dec(0x9D10), Ok(I::SkipIfNotEqual(13, Reg(1))));

        assert_eq!(dec(0x61FC), Ok(I::LoadRegister(1, Lit(0xFC))));
        assert_eq!(dec(0x6401), Ok(I::LoadRegister(4, Lit(1))));
        assert_eq!(dec(0x69FF), Ok(I::LoadRegister(9, Lit(0xFF))));
        assert_eq!(dec(0x6E0E), Ok(I::LoadRegister(14, Lit(14))));

        assert_eq!(dec(0x728D), Ok(I::AddNoCarry(2, 0x8D)));
        assert_eq!(dec(0x7A56), Ok(I::AddNoCarry(10, 0x56)));
        assert_eq!(dec(0x740F), Ok(I::AddNoCarry(4, 15)));
        assert_eq!(dec(0x7019), Ok(I::AddNoCarry(0, 0x19)));

        assert_eq!(dec(0x8010), Ok(I::LoadRegister(0, Reg(1))));
        assert_eq!(dec(0x8490), Ok(I::LoadRegister(4, Reg(9))));
        assert_eq!(dec(0x8800), Ok(I::LoadRegister(8, Reg(0))));
        assert_eq!(dec(0x8CA0), Ok(I::LoadRegister(12, Reg(10))));
        assert_eq!(dec(0x8F20), Ok(I::LoadRegister(15, Reg(2))));
        assert_eq!(dec(0x83C0), Ok(I::LoadRegister(3, Reg(12))));

        assert_eq!(dec(0x8011), Ok(I::Or(0, 1)));
        assert_eq!(dec(0x8491), Ok(I::Or(4, 9)));
        assert_eq!(dec(0x8801), Ok(I::Or(8, 0)));
        assert_eq!(dec(0x8CA1), Ok(I::Or(12, 10)));
        assert_eq!(dec(0x8F21), Ok(I::Or(15, 2)));
        assert_eq!(dec(0x83C1), Ok(I::Or(3, 12)));

        assert_eq!(dec(0x8012), Ok(I::And(0, 1)));
        assert_eq!(dec(0x8492), Ok(I::And(4, 9)));
        assert_eq!(dec(0x8802), Ok(I::And(8, 0)));
        assert_eq!(dec(0x8CA2), Ok(I::And(12, 10)));
        assert_eq!(dec(0x8F22), Ok(I::And(15, 2)));
        assert_eq!(dec(0x83C2), Ok(I::And(3, 12)));

        assert_eq!(dec(0x8013), Ok(I::Xor(0, 1)));
        assert_eq!(dec(0x8493), Ok(I::Xor(4, 9)));
        assert_eq!(dec(0x8803), Ok(I::Xor(8, 0)));
        assert_eq!(dec(0x8CA3), Ok(I::Xor(12, 10)));
        assert_eq!(dec(0x8F23), Ok(I::Xor(15, 2)));
        assert_eq!(dec(0x83C3), Ok(I::Xor(3, 12)));

        assert_eq!(dec(0x8014), Ok(I::AddWithCarry(0, 1)));
        assert_eq!(dec(0x8494), Ok(I::AddWithCarry(4, 9)));
        assert_eq!(dec(0x8804), Ok(I::AddWithCarry(8, 0)));
        assert_eq!(dec(0x8CA4), Ok(I::AddWithCarry(12, 10)));
        assert_eq!(dec(0x8F24), Ok(I::AddWithCarry(15, 2)));
        assert_eq!(dec(0x83C4), Ok(I::AddWithCarry(3, 12)));

        assert_eq!(dec(0x8015), Ok(I::Sub(0, 1)));
        assert_eq!(dec(0x8495), Ok(I::Sub(4, 9)));
        assert_eq!(dec(0x8805), Ok(I::Sub(8, 0)));
        assert_eq!(dec(0x8CA5), Ok(I::Sub(12, 10)));
        assert_eq!(dec(0x8F25), Ok(I::Sub(15, 2)));
        assert_eq!(dec(0x83C5), Ok(I::Sub(3, 12)));

        assert_eq!(dec(0x8016), Ok(I::ShiftRight(0)));
        assert_eq!(dec(0x8496), Ok(I::ShiftRight(4)));
        assert_eq!(dec(0x8806), Ok(I::ShiftRight(8)));
        assert_eq!(dec(0x8CA6), Ok(I::ShiftRight(12)));
        assert_eq!(dec(0x8F26), Ok(I::ShiftRight(15)));
        assert_eq!(dec(0x83C6), Ok(I::ShiftRight(3)));

        assert_eq!(dec(0x8017), Ok(I::SubN(0, 1)));
        assert_eq!(dec(0x8497), Ok(I::SubN(4, 9)));
        assert_eq!(dec(0x8807), Ok(I::SubN(8, 0)));
        assert_eq!(dec(0x8CA7), Ok(I::SubN(12, 10)));
        assert_eq!(dec(0x8F27), Ok(I::SubN(15, 2)));
        assert_eq!(dec(0x83C7), Ok(I::SubN(3, 12)));

        assert_eq!(dec(0x801E), Ok(I::ShiftLeft(0)));
        assert_eq!(dec(0x849E), Ok(I::ShiftLeft(4)));
        assert_eq!(dec(0x880E), Ok(I::ShiftLeft(8)));
        assert_eq!(dec(0x8CAE), Ok(I::ShiftLeft(12)));
        assert_eq!(dec(0x8F2E), Ok(I::ShiftLeft(15)));
        assert_eq!(dec(0x83CE), Ok(I::ShiftLeft(3)));

        assert_eq!(dec(0xA375), Ok(I::LoadMemoryRegister(0x375)));
        assert_eq!(dec(0xA200), Ok(I::LoadMemoryRegister(0x200)));
        assert_eq!(dec(0xA9FD), Ok(I::LoadMemoryRegister(0x9FD)));
        assert_eq!(dec(0xAA42), Ok(I::LoadMemoryRegister(0xA42)));

        assert_eq!(dec(0xB375), Ok(I::JumpPlusV0(0x375)));
        assert_eq!(dec(0xB200), Ok(I::JumpPlusV0(0x200)));
        assert_eq!(dec(0xB9FD), Ok(I::JumpPlusV0(0x9FD)));
        assert_eq!(dec(0xBA42), Ok(I::JumpPlusV0(0xA42)));

        assert_eq!(dec(0xC234), Ok(I::LoadRandomWithMask(2, 0x34)));
        assert_eq!(dec(0xC000), Ok(I::LoadRandomWithMask(0, 0x00)));
        assert_eq!(dec(0xC4FF), Ok(I::LoadRandomWithMask(4, 0xFF)));
        assert_eq!(dec(0xCEAA), Ok(I::LoadRandomWithMask(14, 0xAA)));

        assert_eq!(dec(0xD015), Ok(I::Draw(0, 1, 5)));
        assert_eq!(dec(0xD409), Ok(I::Draw(4, 0, 9)));
        assert_eq!(dec(0xD782), Ok(I::Draw(7, 8, 2)));
        assert_eq!(dec(0xD47D), Ok(I::Draw(4, 7, 13)));
        assert_eq!(dec(0xD66F), Ok(I::Draw(6, 6, 15)));
        assert_eq!(dec(0xDE4A), Ok(I::Draw(14, 4, 10)));

        assert_eq!(dec(0xE09E), Ok(I::SkipIfKeyPressed(0)));
        assert_eq!(dec(0xE49E), Ok(I::SkipIfKeyPressed(4)));
        assert_eq!(dec(0xE99E), Ok(I::SkipIfKeyPressed(9)));
        assert_eq!(dec(0xEB9E), Ok(I::SkipIfKeyPressed(11)));

        assert_eq!(dec(0xE0A1), Ok(I::SkipIfKeyNotPressed(0)));
        assert_eq!(dec(0xE4A1), Ok(I::SkipIfKeyNotPressed(4)));
        assert_eq!(dec(0xE9A1), Ok(I::SkipIfKeyNotPressed(9)));
        assert_eq!(dec(0xEBA1), Ok(I::SkipIfKeyNotPressed(11)));

        assert_eq!(dec(0xF107), Ok(I::LoadFromDelayTimer(1)));
        assert_eq!(dec(0xF307), Ok(I::LoadFromDelayTimer(3)));
        assert_eq!(dec(0xF607), Ok(I::LoadFromDelayTimer(6)));
        assert_eq!(dec(0xF807), Ok(I::LoadFromDelayTimer(8)));
        assert_eq!(dec(0xFC07), Ok(I::LoadFromDelayTimer(12)));
        assert_eq!(dec(0xFE07), Ok(I::LoadFromDelayTimer(14)));

        assert_eq!(dec(0xF10A), Ok(I::WaitForKeyPress(1)));
        assert_eq!(dec(0xF30A), Ok(I::WaitForKeyPress(3)));
        assert_eq!(dec(0xF60A), Ok(I::WaitForKeyPress(6)));
        assert_eq!(dec(0xF80A), Ok(I::WaitForKeyPress(8)));
        assert_eq!(dec(0xFC0A), Ok(I::WaitForKeyPress(12)));
        assert_eq!(dec(0xFE0A), Ok(I::WaitForKeyPress(14)));

        assert_eq!(dec(0xF115), Ok(I::LoadIntoDelayTimer(1)));
        assert_eq!(dec(0xF315), Ok(I::LoadIntoDelayTimer(3)));
        assert_eq!(dec(0xF615), Ok(I::LoadIntoDelayTimer(6)));
        assert_eq!(dec(0xF815), Ok(I::LoadIntoDelayTimer(8)));
        assert_eq!(dec(0xFC15), Ok(I::LoadIntoDelayTimer(12)));
        assert_eq!(dec(0xFE15), Ok(I::LoadIntoDelayTimer(14)));

        assert_eq!(dec(0xF118), Ok(I::LoadIntoSoundTimer(1)));
        assert_eq!(dec(0xF318), Ok(I::LoadIntoSoundTimer(3)));
        assert_eq!(dec(0xF618), Ok(I::LoadIntoSoundTimer(6)));
        assert_eq!(dec(0xF818), Ok(I::LoadIntoSoundTimer(8)));
        assert_eq!(dec(0xFC18), Ok(I::LoadIntoSoundTimer(12)));
        assert_eq!(dec(0xFE18), Ok(I::LoadIntoSoundTimer(14)));

        assert_eq!(dec(0xF11E), Ok(I::AddToMemoryRegister(1)));
        assert_eq!(dec(0xF31E), Ok(I::AddToMemoryRegister(3)));
        assert_eq!(dec(0xF61E), Ok(I::AddToMemoryRegister(6)));
        assert_eq!(dec(0xF81E), Ok(I::AddToMemoryRegister(8)));
        assert_eq!(dec(0xFC1E), Ok(I::AddToMemoryRegister(12)));
        assert_eq!(dec(0xFE1E), Ok(I::AddToMemoryRegister(14)));

        assert_eq!(dec(0xF129), Ok(I::LoadDigitAddress(1)));
        assert_eq!(dec(0xF329), Ok(I::LoadDigitAddress(3)));
        assert_eq!(dec(0xF629), Ok(I::LoadDigitAddress(6)));
        assert_eq!(dec(0xF829), Ok(I::LoadDigitAddress(8)));
        assert_eq!(dec(0xFC29), Ok(I::LoadDigitAddress(12)));
        assert_eq!(dec(0xFE29), Ok(I::LoadDigitAddress(14)));

        assert_eq!(dec(0xF133), Ok(I::StoreBcdInMemory(1)));
        assert_eq!(dec(0xF333), Ok(I::StoreBcdInMemory(3)));
        assert_eq!(dec(0xF633), Ok(I::StoreBcdInMemory(6)));
        assert_eq!(dec(0xF833), Ok(I::StoreBcdInMemory(8)));
        assert_eq!(dec(0xFC33), Ok(I::StoreBcdInMemory(12)));
        assert_eq!(dec(0xFE33), Ok(I::StoreBcdInMemory(14)));

        assert_eq!(dec(0xF155), Ok(I::StoreRegistersInMemory(1)));
        assert_eq!(dec(0xF355), Ok(I::StoreRegistersInMemory(3)));
        assert_eq!(dec(0xF655), Ok(I::StoreRegistersInMemory(6)));
        assert_eq!(dec(0xF855), Ok(I::StoreRegistersInMemory(8)));
        assert_eq!(dec(0xFC55), Ok(I::StoreRegistersInMemory(12)));
        assert_eq!(dec(0xFE55), Ok(I::StoreRegistersInMemory(14)));

        assert_eq!(dec(0xF165), Ok(I::ReadRegistersFromMemory(1)));
        assert_eq!(dec(0xF365), Ok(I::ReadRegistersFromMemory(3)));
        assert_eq!(dec(0xF665), Ok(I::ReadRegistersFromMemory(6)));
        assert_eq!(dec(0xF865), Ok(I::ReadRegistersFromMemory(8)));
        assert_eq!(dec(0xFC65), Ok(I::ReadRegistersFromMemory(12)));
        assert_eq!(dec(0xFE65), Ok(I::ReadRegistersFromMemory(14)));
    }

    #[test]
    fn decode_error_test() {
        assert_eq!(
            dec(0xFFFF),
            Err(DecodingError::UnrecognisedBytecode(0xFFFF))
        );
        assert_eq!(
            dec(0x5931),
            Err(DecodingError::UnrecognisedBytecode(0x5931))
        );
        assert_eq!(
            dec(0x5C09),
            Err(DecodingError::UnrecognisedBytecode(0x5C09))
        );
        assert_eq!(
            dec(0x89DA),
            Err(DecodingError::UnrecognisedBytecode(0x89DA))
        );
        assert_eq!(
            dec(0x8FFF),
            Err(DecodingError::UnrecognisedBytecode(0x8FFF))
        );
        assert_eq!(
            dec(0x00CD),
            Err(DecodingError::UnrecognisedBytecode(0x00CD))
        );
        assert_eq!(
            dec(0xEE09),
            Err(DecodingError::UnrecognisedBytecode(0xEE09))
        );
        assert_eq!(
            dec(0xE17C),
            Err(DecodingError::UnrecognisedBytecode(0xE17C))
        );
    }
}
