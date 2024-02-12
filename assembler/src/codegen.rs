//! This module handles the final stage before encoding the instructions to bytecode. We just have
//! to resolve alias definitions.

use crate::{
    ast::{AliasableThing, OrAlias, PseudoInstruction as PI, RegOrByte, Stmt},
    error::report_error,
    span::WithSpan,
};
use chip8_instructions::{encode, EncodingError, Instruction as I, Operand};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum CodegenError<'s> {
    #[error("The alias {0:?} was already defined")]
    AliasAlreadyDefined(&'s str),

    #[error("The label {0:?} was already defined")]
    LabelAlreadyDefined(&'s str),

    #[error("The alias {0:?} is not defined")]
    AliasNotDefined(&'s str),

    #[error("The alias {0:?} should be a register but isn't")]
    AliasShouldBeRegister(&'s str),

    #[error("The alias {0:?} should be a raw number but isn't")]
    AliasShouldBeNumber(&'s str),

    #[error("Failed to encode instr: {0:?}")]
    EncodingError(#[from] EncodingError),

    #[error("Alias {0:?} resolved to a number which was too large: {1} should be at most {2}")]
    AliasedLiteralTooBig(&'s str, u16, u16),
}

/// Resolve all the defined aliases and labels to produce a list of instructions ready to encode.
///
/// This method currently emits an error and moves on if it encounters a [`Stmt::Include`] directive.
pub fn codegen<'s>(
    statements: Vec<WithSpan<Stmt<'s>>>,
) -> Result<Vec<u8>, WithSpan<CodegenError<'s>>> {
    // The first pass is just to get numbers for all the aliases.
    let mut offset: u16 = 0x200;
    let mut alias_map: HashMap<&'s str, AliasableThing> = HashMap::new();

    for WithSpan { span, value: stmt } in statements.iter() {
        match stmt {
            Stmt::AliasDefinition(name, thing) => {
                if alias_map.insert(name, *thing).is_some() {
                    return Err(WithSpan {
                        value: CodegenError::AliasAlreadyDefined(name),
                        span: *span,
                    });
                }
            }
            Stmt::RawDataDefinition(data) => offset += data.len() as u16,
            Stmt::Label(name) => {
                if alias_map
                    .insert(name, AliasableThing::RawData(offset))
                    .is_some()
                {
                    return Err(WithSpan {
                        value: CodegenError::LabelAlreadyDefined(name),
                        span: *span,
                    });
                }
            }
            Stmt::PseudoInstruction(_) => offset += 2,
            Stmt::Include(_) => report_error(
                *span,
                "Including other files is currently not implemented, so this will be ignored",
            ),
        };
    }

    let mut blob: Vec<u8> = Vec::with_capacity(offset as usize - 0x200);

    for WithSpan { span, value: stmt } in statements.into_iter() {
        macro_rules! resolve_addr {
            ($arg:ident) => {
                match $arg {
                    OrAlias::Alias(alias) => match *alias_map.get(alias).ok_or(WithSpan {
                        value: CodegenError::AliasNotDefined(alias),
                        span,
                    })? {
                        AliasableThing::RawData(data) => data,
                        AliasableThing::Register(_) => {
                            return Err(WithSpan {
                                value: CodegenError::AliasShouldBeNumber(alias),
                                span,
                            });
                        }
                    },
                    OrAlias::Concrete(addr) => addr,
                }
            };
        }

        macro_rules! resolve_reg {
            ($arg:ident) => {
                match $arg {
                    OrAlias::Alias(alias) => match *alias_map.get(alias).ok_or(WithSpan {
                        value: CodegenError::AliasNotDefined(alias),
                        span,
                    })? {
                        AliasableThing::RawData(_) => {
                            return Err(WithSpan {
                                value: CodegenError::AliasShouldBeRegister(alias),
                                span,
                            });
                        }
                        AliasableThing::Register(register) => register as u8,
                    },
                    OrAlias::Concrete(reg) => reg as u8,
                }
            };
        }

        macro_rules! resolve_reg_or_byte {
            ($arg:ident; $reg_name:ident => $reg_code:expr; $byte_name:ident => $byte_code:expr) => {
                match $arg {
                    OrAlias::Alias(alias) => match *alias_map.get(alias).ok_or(WithSpan {
                        value: CodegenError::AliasNotDefined(alias),
                        span,
                    })? {
                        AliasableThing::RawData(data) => {
                            if data > 0xFF {
                                return Err(WithSpan {
                                    value: CodegenError::AliasedLiteralTooBig(alias, data, 0xFF),
                                    span,
                                });
                            }
                            let $byte_name = data as u8;
                            $byte_code
                        }
                        AliasableThing::Register($reg_name) => $reg_code,
                    },
                    OrAlias::Concrete(RegOrByte::Register($reg_name)) => $reg_code,
                    OrAlias::Concrete(RegOrByte::LiteralByte($byte_name)) => $byte_code,
                }
            };
        }

        match stmt {
            Stmt::AliasDefinition(_, _) | Stmt::Label(_) => {}
            Stmt::RawDataDefinition(data) => blob.extend(data),
            Stmt::PseudoInstruction(instr) => {
                let instruction = match instr {
                    PI::Nop => I::Nop,
                    PI::Cls => I::ClearScreen,
                    PI::Ret => I::Return,
                    PI::Jmp(addr) => I::Jump(resolve_addr!(addr)),
                    PI::JmpPlus(reg, addr) => {
                        let reg = resolve_reg!(reg);
                        if reg != 0 {
                            report_error(
                                span,
                                "The jmpp instruction only supports jumping plus V0",
                            );
                            panic!("The jmpp instruction only supports jumping plus V0");
                        }
                        let addr = resolve_addr!(addr);
                        I::JumpPlusV0(addr)
                    }
                    PI::Call(addr) => I::Call(resolve_addr!(addr)),
                    PI::Se(reg, reg_or_byte) => {
                        let r1 = resolve_reg!(reg);
                        resolve_reg_or_byte!(
                            reg_or_byte;
                            r2 => I::SkipIfEqual(r1, Operand::Register(r2 as u8));
                            data => I::SkipIfEqual(r1, Operand::Literal(data))
                        )
                    }
                    PI::Sne(reg, reg_or_byte) => {
                        let r1 = resolve_reg!(reg);
                        resolve_reg_or_byte!(
                            reg_or_byte;
                            r2 => I::SkipIfNotEqual(r1, Operand::Register(r2 as u8));
                            data => I::SkipIfNotEqual(r1, Operand::Literal(data))
                        )
                    }
                    PI::Ld(reg, reg_or_byte) => {
                        let r1 = resolve_reg!(reg);
                        resolve_reg_or_byte!(
                            reg_or_byte;
                            r2 => I::LoadRegister(r1, Operand::Register(r2 as u8));
                            data => I::LoadRegister(r1, Operand::Literal(data))
                        )
                    }
                    PI::LdIndex(addr) => I::LoadMemoryRegister(resolve_addr!(addr)),
                    PI::LdFromK(reg) => I::WaitForKeyPress(resolve_reg!(reg)),
                    PI::LdFromDt(reg) => I::LoadFromDelayTimer(resolve_reg!(reg)),
                    PI::Add(r1, reg_or_byte) => {
                        let r1 = resolve_reg!(r1);
                        resolve_reg_or_byte!(
                            reg_or_byte;
                            r2 => I::AddWithCarry(r1, r2 as u8);
                            data => I::AddNoCarry(r1, data)
                        )
                    }
                    PI::AddIndex(reg) => I::AddToMemoryRegister(resolve_reg!(reg)),
                    PI::Or(r1, r2) => I::Or(resolve_reg!(r1), resolve_reg!(r2)),
                    PI::And(r1, r2) => I::And(resolve_reg!(r1), resolve_reg!(r2)),
                    PI::Xor(r1, r2) => I::Xor(resolve_reg!(r1), resolve_reg!(r2)),
                    PI::Sub(r1, r2) => I::Sub(resolve_reg!(r1), resolve_reg!(r2)),
                    PI::Subn(r1, r2) => I::SubN(resolve_reg!(r1), resolve_reg!(r2)),
                    PI::Shr(reg) => I::ShiftRight(resolve_reg!(reg)),
                    PI::Shl(reg) => I::ShiftLeft(resolve_reg!(reg)),
                    PI::Rnd(reg, mask) => {
                        let reg = resolve_reg!(reg);
                        let mask = match mask {
                            OrAlias::Alias(alias) => {
                                match *alias_map.get(alias).ok_or(WithSpan {
                                    value: CodegenError::AliasNotDefined(alias),
                                    span,
                                })? {
                                    AliasableThing::RawData(data) => {
                                        if data > 0xFF {
                                            return Err(WithSpan {
                                                value: CodegenError::AliasedLiteralTooBig(
                                                    alias, data, 0xFF,
                                                ),
                                                span,
                                            });
                                        }
                                        data as u8
                                    }
                                    AliasableThing::Register(_) => {
                                        return Err(WithSpan {
                                            value: CodegenError::AliasShouldBeNumber(alias),
                                            span,
                                        });
                                    }
                                }
                            }
                            OrAlias::Concrete(byte) => byte,
                        };
                        I::LoadRandomWithMask(reg, mask)
                    }
                    PI::Drw(r1, r2, nibble) => {
                        let r1 = resolve_reg!(r1);
                        let r2 = resolve_reg!(r2);
                        let nibble = match nibble {
                            OrAlias::Alias(alias) => {
                                match *alias_map.get(alias).ok_or(WithSpan {
                                    value: CodegenError::AliasNotDefined(alias),
                                    span,
                                })? {
                                    AliasableThing::RawData(data) => {
                                        if data > 0xF {
                                            return Err(WithSpan {
                                                value: CodegenError::AliasedLiteralTooBig(
                                                    alias, data, 0xF,
                                                ),
                                                span,
                                            });
                                        }
                                        data as u8
                                    }
                                    AliasableThing::Register(_) => {
                                        return Err(WithSpan {
                                            value: CodegenError::AliasShouldBeNumber(alias),
                                            span,
                                        });
                                    }
                                }
                            }
                            OrAlias::Concrete(byte) => byte,
                        };
                        I::Draw(r1, r2, nibble)
                    }
                    PI::Skp(reg) => I::SkipIfKeyPressed(resolve_reg!(reg)),
                    PI::Sknp(reg) => I::SkipIfKeyNotPressed(resolve_reg!(reg)),
                    PI::Delay(reg) => I::LoadIntoDelayTimer(resolve_reg!(reg)),
                    PI::Sound(reg) => I::LoadIntoSoundTimer(resolve_reg!(reg)),
                    PI::Font(reg) => I::LoadDigitAddress(resolve_reg!(reg)),
                    PI::Bcd(reg) => I::StoreBcdInMemory(resolve_reg!(reg)),
                    PI::Stor(reg) => I::StoreRegistersInMemory(resolve_reg!(reg)),
                    PI::Rstr(reg) => I::ReadRegistersFromMemory(resolve_reg!(reg)),
                };
                blob.extend(encode(instruction).map_err(|encoding_error| WithSpan {
                    value: CodegenError::EncodingError(encoding_error),
                    span,
                })?);
            }
            Stmt::Include(_) => {} // We already emitted an error on the first pass
        }
    }

    Ok(blob)
}
