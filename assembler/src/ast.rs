//! This module handles the AST.

#![allow(dead_code)]

use crate::{span::WithSpan, tokens::GeneralRegisterName};

/// Something that can be aliased.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasableThing {
    RawData(u16),
    Register(GeneralRegisterName),
}

/// Either an argument to an instruction, or an alias.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrAlias<'s, T>
where
    T: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
{
    Alias(&'s str),
    Concrete(T),
}

impl<'s, T> OrAlias<'s, T>
where
    T: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
{
    pub fn map<U, F>(self, func: F) -> OrAlias<'s, U>
    where
        U: Clone + Copy + std::fmt::Debug + PartialEq + Eq,
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Alias(alias) => OrAlias::Alias(alias),
            Self::Concrete(t) => OrAlias::Concrete(func(t)),
        }
    }
}

/// A u8 or an alias.
type Byte<'s> = OrAlias<'s, u8>;

/// A u16 or an alias.
type Word<'s> = OrAlias<'s, u16>;

/// A general register or an alias.
type Reg<'s> = OrAlias<'s, GeneralRegisterName>;

/// A register or a literal byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegOrByte {
    Register(GeneralRegisterName),
    LiteralByte(u8),
}

/// A register, literal byte, or an alias.
type RegOrByteA<'s> = OrAlias<'s, RegOrByte>;

/// A pseudo-instruction, which is almost a real instruction, but it still needs an aliasing pass
/// to resolve any defines or ambiguities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PseudoInstruction<'s> {
    Nop,
    Cls,
    Ret,
    Jmp(Word<'s>),
    JmpPlus(Reg<'s>, Word<'s>),
    Call(Word<'s>),
    Se(Reg<'s>, RegOrByteA<'s>),
    Sne(Reg<'s>, RegOrByteA<'s>),
    Ld(Reg<'s>, RegOrByteA<'s>),
    LdIndex(Word<'s>),
    LdFromK(Reg<'s>),
    LdFromDt(Reg<'s>),
    Add(Reg<'s>, RegOrByteA<'s>),
    AddIndex(Reg<'s>),
    Or(Reg<'s>, Reg<'s>),
    And(Reg<'s>, Reg<'s>),
    Xor(Reg<'s>, Reg<'s>),
    Sub(Reg<'s>, Reg<'s>),
    Subn(Reg<'s>, Reg<'s>),
    Shr(Reg<'s>),
    Shl(Reg<'s>),
    Rnd(Reg<'s>, Byte<'s>),
    Drw(Reg<'s>, Reg<'s>, Byte<'s>),
    Skp(Reg<'s>),
    Sknp(Reg<'s>),
    Delay(Reg<'s>),
    Sound(Reg<'s>),
    Font(Reg<'s>),
    Bcd(Reg<'s>),
    Stor(Reg<'s>),
    Rstr(Reg<'s>),
}

/// A [`Stmt`] wrapped in [`WithSpan`].
pub type SpanStmt<'s> = WithSpan<Stmt<'s>>;

/// A list of all the possible statements.
#[derive(Clone, Debug, PartialEq)]
pub enum Stmt<'s> {
    AliasDefinition(&'s str, AliasableThing),
    RawDataDefinition(Vec<u8>),
    Label(&'s str),
    PseudoInstruction(PseudoInstruction<'s>),
    Include(&'s str),
}
