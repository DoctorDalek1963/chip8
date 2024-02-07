//! This module contains token definitions.

use crate::span::WithSpan;

pub type TokenSpan<'s> = WithSpan<Token<'s>>;

/// A list of all the tokens supported by this CHIP-8 assembly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Token<'s> {
    Colon,
    Identifier(&'s str),
    InstructionName(InstructionName),
    GeneralRegisterName(GeneralRegisterName),
    SpecialRegisterName(SpecialRegisterName),
    Define,
    DefineBytes,
    DefineWords,
    NumericLiteral(u16),
    Include,
    StringLiteral(&'s str),
    Text,
}

/// All the instruction mnemonics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InstructionName {
    Nop,
    Cls,
    Ret,
    Jmp,
    Jmpp,
    Call,
    Se,
    Sne,
    Ld,
    Add,
    Or,
    And,
    Xor,
    Sub,
    Subn,
    Shr,
    Shl,
    Rnd,
    Drw,
    Skp,
    Sknp,
    Delay,
    Sound,
    Font,
    Bcd,
    Stor,
    Rstr,
}

/// All the names of the general registers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GeneralRegisterName {
    V0 = 0,
    V1 = 1,
    V2 = 2,
    V3 = 3,
    V4 = 4,
    V5 = 5,
    V6 = 6,
    V7 = 7,
    V8 = 8,
    V9 = 9,
    Va = 10,
    Vb = 11,
    Vc = 12,
    Vd = 13,
    Ve = 14,
    Vf = 15,
}

/// The special registers used in mnemonics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpecialRegisterName {
    /// The memory register, or "index".
    I,

    /// The delay timer.
    Dt,

    /// The keyboard. Only used to wait for a keypress.
    K,
}
