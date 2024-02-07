//! This module handles parsing instructions.

use super::{ParseError, ParseResult, Parser};
use crate::{
    ast::{OrAlias, PseudoInstruction as PI, RegOrByte, SpanStmt, Stmt},
    span::{Span, WithSpan},
    tokens::{GeneralRegisterName, InstructionName as IN, SpecialRegisterName, Token as T},
};

impl<'s> Parser<'s> {
    /// See the instruction module for details.
    pub(super) fn parse_instruction(&mut self) -> ParseResult<'s, SpanStmt<'s>> {
        let WithSpan {
            span: instr_span,
            value: T::InstructionName(instr_name),
        } = self.advance()
        else {
            panic!("We should only call parse_instruction() when the previous token is an instruction name");
        };

        macro_rules! one_reg {
            ($pseudo:ident) => {{
                let (reg, span) = self.parse_arg_general_register(instr_span)?;
                (PI::$pseudo(reg), Some(span))
            }};
        }

        macro_rules! two_reg {
            ($pseudo:ident) => {{
                let (r1, r1_span) = self.parse_arg_general_register(instr_span)?;
                let (r2, r2_span) = self.parse_arg_general_register(instr_span.union(&r1_span))?;
                (PI::$pseudo(r1, r2), Some(instr_span.union(&r2_span)))
            }};
        }

        macro_rules! reg_or_byte {
            ($r1_span:expr) => {
                match self.parse_arg_general_register(instr_span.union(&($r1_span))) {
                    Ok((r2, r2_span)) => (r2.map(RegOrByte::Register), r2_span),
                    Err(_) => {
                        self.step_back();
                        let (byte, byte_span) = self.parse_arg_byte(instr_span)?;
                        (byte.map(RegOrByte::LiteralByte), byte_span)
                    }
                }
            };
        }

        let (pseudo_instr, args_span) = match instr_name {
            IN::Nop => (PI::Nop, None),
            IN::Cls => (PI::Cls, None),
            IN::Ret => (PI::Ret, None),
            IN::Jmp => {
                let (addr, span) = self.parse_arg_addr(instr_span)?;
                (PI::Jmp(addr), Some(span))
            }
            IN::Jmpp => {
                let (reg, reg_span) = self.parse_arg_general_register(instr_span)?;
                let (addr, addr_span) = self.parse_arg_addr(instr_span.union(&reg_span))?;
                (PI::JmpPlus(reg, addr), Some(reg_span.union(&addr_span)))
            }
            IN::Call => {
                let (addr, span) = self.parse_arg_addr(instr_span)?;
                (PI::Call(addr), Some(span))
            }
            IN::Se => {
                let (r1, r1_span) = self.parse_arg_general_register(instr_span)?;
                let (arg2, arg2_span) = reg_or_byte!(r1_span);
                (PI::Se(r1, arg2), Some(r1_span.union(&arg2_span)))
            }
            IN::Sne => {
                let (r1, r1_span) = self.parse_arg_general_register(instr_span)?;
                let (arg2, arg2_span) = reg_or_byte!(r1_span);
                (PI::Sne(r1, arg2), Some(r1_span.union(&arg2_span)))
            }
            IN::Ld => self
                .parse_load(instr_span)
                .map(|(pi, span)| (pi, Some(span)))?,
            IN::Add => match self.peek() {
                Some(&WithSpan {
                    span,
                    value: T::SpecialRegisterName(SpecialRegisterName::I),
                }) => {
                    self.advance();
                    let (r2, r2_span) = self.parse_arg_general_register(instr_span.union(&span))?;
                    (PI::AddIndex(r2), Some(span.union(&r2_span)))
                }
                _ => {
                    let (r1, r1_span) = self.parse_arg_general_register(instr_span)?;
                    let (arg2, arg2_span) = reg_or_byte!(r1_span);
                    (PI::Add(r1, arg2), Some(r1_span.union(&arg2_span)))
                }
            },
            IN::Or => two_reg!(Or),
            IN::And => two_reg!(And),
            IN::Xor => two_reg!(Xor),
            IN::Sub => two_reg!(Sub),
            IN::Subn => two_reg!(Subn),
            IN::Rnd => {
                let (reg, reg_span) = self.parse_arg_general_register(instr_span)?;
                let (byte, byte_span) = self.parse_arg_byte(instr_span.union(&reg_span))?;
                (PI::Rnd(reg, byte), Some(reg_span.union(&byte_span)))
            }
            IN::Drw => {
                let (r1, r1_span) = self.parse_arg_general_register(instr_span)?;
                let (r2, r2_span) = self.parse_arg_general_register(instr_span.union(&r1_span))?;
                let (nibble, nibble_span) = self.parse_arg_nibble(instr_span.union(&r2_span))?;
                (
                    PI::Drw(r1, r2, nibble),
                    Some(instr_span.union(&nibble_span)),
                )
            }
            IN::Shr => one_reg!(Shr),
            IN::Shl => one_reg!(Shl),
            IN::Skp => one_reg!(Skp),
            IN::Sknp => one_reg!(Sknp),
            IN::Delay => one_reg!(Delay),
            IN::Sound => one_reg!(Sound),
            IN::Font => one_reg!(Font),
            IN::Bcd => one_reg!(Bcd),
            IN::Stor => one_reg!(Stor),
            IN::Rstr => one_reg!(Rstr),
        };

        let span = if let Some(span) = args_span {
            instr_span.union(&span)
        } else {
            instr_span
        };

        Ok(WithSpan {
            span,
            value: Stmt::PseudoInstruction(pseudo_instr),
        })
    }

    fn parse_arg_nibble(
        &mut self,
        previous_span: Span,
    ) -> ParseResult<'s, (OrAlias<'s, u8>, Span)> {
        let token = self.advance();
        match token.value {
            T::Identifier(name) => Ok((OrAlias::Alias(name), token.span)),
            T::NumericLiteral(num) if num <= 15 => Ok((OrAlias::Concrete(num as u8), token.span)),
            T::NumericLiteral(num) if num > 15 => Err(ParseError {
                token,
                previous_span: Some(previous_span),
                message: "Numeric literal too large for argument which was expected to be 1 nibble"
                    .to_string(),
            }),
            _ => Err(ParseError {
                token,
                previous_span: Some(previous_span),
                message: "Expected alias or numeric literal (nibble) for this argument".to_string(),
            }),
        }
    }

    fn parse_arg_byte(&mut self, previous_span: Span) -> ParseResult<'s, (OrAlias<'s, u8>, Span)> {
        let token = self.advance();
        match token.value {
            T::Identifier(name) => Ok((OrAlias::Alias(name), token.span)),
            T::NumericLiteral(num) if num <= 255 => Ok((OrAlias::Concrete(num as u8), token.span)),
            T::NumericLiteral(num) if num > 255 => Err(ParseError {
                token,
                previous_span: Some(previous_span),
                message: "Numeric literal too large for argument which was expected to be 1 byte"
                    .to_string(),
            }),
            _ => Err(ParseError {
                token,
                previous_span: Some(previous_span),
                message: "Expected alias or numeric literal (byte) for this argument".to_string(),
            }),
        }
    }

    fn parse_arg_addr(&mut self, previous_span: Span) -> ParseResult<'s, (OrAlias<'s, u16>, Span)> {
        let token = self.advance();
        match token.value {
            T::Identifier(name) => Ok((OrAlias::Alias(name), token.span)),
            T::NumericLiteral(num) if num <= 0xFFF => Ok((OrAlias::Concrete(num), token.span)),
            T::NumericLiteral(num) if num > 0xFFF => Err(ParseError {
                token,
                previous_span: Some(previous_span),
                message: "Numeric literal too large for argument which was expected to be 12 bits"
                    .to_string(),
            }),
            _ => Err(ParseError {
                token,
                previous_span: Some(previous_span),
                message: "Expected alias or numeric literal (12-bit) for this argument".to_string(),
            }),
        }
    }

    fn parse_arg_general_register(
        &mut self,
        previous_span: Span,
    ) -> ParseResult<'s, (OrAlias<'s, GeneralRegisterName>, Span)> {
        let token = self.advance();
        match token.value {
            T::Identifier(name) => Ok((OrAlias::Alias(name), token.span)),
            T::GeneralRegisterName(reg) => Ok((OrAlias::Concrete(reg), token.span)),
            _ => Err(ParseError {
                token,
                previous_span: Some(previous_span),
                message: "Expected alias or general register name for this argument".to_string(),
            }),
        }
    }

    fn parse_load(&mut self, instr_span: Span) -> ParseResult<'s, (PI<'s>, Span)> {
        Ok(match self.peek() {
            Some(&WithSpan {
                span,
                value: T::SpecialRegisterName(SpecialRegisterName::I),
            }) => {
                self.advance();
                let (addr, addr_span) = self.parse_arg_addr(instr_span.union(&span))?;
                (PI::LdIndex(addr), span.union(&addr_span))
            }
            _ => {
                let (r1, r1_span) = self.parse_arg_general_register(instr_span)?;
                match self.peek() {
                    Some(&WithSpan {
                        span,
                        value: T::SpecialRegisterName(SpecialRegisterName::K),
                    }) => {
                        self.advance();
                        (PI::LdFromK(r1), r1_span.union(&span))
                    }
                    Some(&WithSpan {
                        span,
                        value: T::SpecialRegisterName(SpecialRegisterName::Dt),
                    }) => {
                        self.advance();
                        (PI::LdFromDt(r1), r1_span.union(&span))
                    }
                    _ => {
                        let (arg2, arg2_span) =
                            match self.parse_arg_general_register(instr_span.union(&r1_span)) {
                                Ok((r2, r2_span)) => (r2.map(RegOrByte::Register), r2_span),
                                Err(_) => {
                                    self.step_back();
                                    let (byte, byte_span) = self.parse_arg_byte(instr_span)?;
                                    (byte.map(RegOrByte::LiteralByte), byte_span)
                                }
                            };
                        (PI::Ld(r1, arg2), r1_span.union(&arg2_span))
                    }
                }
            }
        })
    }
}
