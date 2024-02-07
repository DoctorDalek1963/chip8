//! This module contains the parsing logic.

mod instruction;

use crate::{
    ast::{AliasableThing, SpanStmt, Stmt},
    error::report_error,
    span::{Span, WithSpan},
    tokens::{self, Token as T, TokenSpan},
};
use core::fmt;
use thiserror::Error;

/// An error that occured during parsing.
#[derive(Clone, Debug, PartialEq, Error)]
struct ParseError<'s> {
    /// The token that caused the error.
    token: WithSpan<tokens::Token<'s>>,

    /// The span of related tokens before this error.
    previous_span: Option<Span>,

    /// The message to display to the user.
    message: String,
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl ParseError<'_> {
    /// Report the parsing error to the user.
    fn report(&self) {
        match self {
            Self {
                token,
                previous_span: Some(span),
                message,
            } => report_error(span.union(&token.span), message),
            Self {
                token,
                previous_span: None,
                message,
            } => report_error(token.span, message),
        }
    }
}

/// A result wrapping a [`ParseError`].
type ParseResult<'s, T, E = ParseError<'s>> = ::std::result::Result<T, E>;

/// A simple recursive descent parser for the assembly.
pub struct Parser<'s> {
    /// The token list that we're parsing.
    tokens: Vec<TokenSpan<'s>>,

    /// The index of the token currently being considered.
    current: usize,

    /// The statements that have been parsed by the parser.
    statements: Vec<SpanStmt<'s>>,
}

impl<'s> Parser<'s> {
    pub fn parse(tokens: Vec<TokenSpan<'s>>) -> Vec<SpanStmt<'s>> {
        let mut parser = Self {
            tokens,
            current: 0,
            statements: vec![],
        };

        parser.parse_program();
        parser.statements
    }

    /// Get the token currently being considered.
    #[inline]
    fn peek(&self) -> Option<&TokenSpan<'s>> {
        self.tokens.get(self.current)
    }

    /// Get the previous token.
    #[inline]
    fn previous(&self) -> Option<&TokenSpan<'s>> {
        self.tokens.get(self.current.saturating_sub(1))
    }

    /// Are we at the end of the tokn list?
    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    /// Advance the internal pointer and get the next token.
    fn advance(&mut self) -> TokenSpan<'s> {
        if !self.is_at_end() {
            self.current += 1;
        }
        *self.previous().unwrap()
    }

    /// Step the internal pointer back by one to reverse the effects of [`Self::advance`].
    fn step_back(&mut self) {
        self.current -= 1;
    }

    /// Synchronize the parser to an assumed correct state after an error.
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            match self.peek() {
                Some(WithSpan { span: _, value }) => match value {
                    T::Identifier(_)
                    | T::InstructionName(_)
                    | T::Define
                    | T::DefineBytes
                    | T::DefineWords
                    | T::Text
                    | T::Include => return,
                    _ => {}
                },
                _ => {}
            }

            self.advance();
        }
    }

    /// program → statement*;
    fn parse_program(&mut self) {
        while !self.is_at_end() {
            if let Some(stmt) = self.parse_statement() {
                self.statements.push(stmt);
            }
        }
    }

    /// statement → aliasDefinition | RawDataDefinition | label | instruction | include;
    fn parse_statement(&mut self) -> Option<SpanStmt<'s>> {
        let result = match self.peek()?.value {
            T::Define => self.parse_alias_definition(),
            T::DefineBytes | T::DefineWords | T::Text => self.parse_raw_data_definition(),
            T::Identifier(_) => self.parse_label(),
            T::InstructionName(_) => self.parse_instruction(),
            T::Include => self.parse_include(),
            _ => Err(ParseError {
                token: *self.peek()?,
                previous_span: None,
                message: "Invalid start of statement".to_string(),
            }),
        };

        match result {
            Ok(stmt) => Some(stmt),
            Err(error) => {
                error.report();
                self.synchronize();
                None
            }
        }
    }

    /// aliasDefinition → "define" IDENTIFIER ALIASABLE_THING;
    fn parse_alias_definition(&mut self) -> ParseResult<'s, SpanStmt<'s>> {
        let WithSpan {
            span: define_span,
            value: T::Define,
        } = self.advance()
        else {
            panic!(
                "We should only call parse_alias_definition() when the previous token is Define"
            );
        };

        let next_token = self.advance();
        let WithSpan {
            span: ident_span,
            value: T::Identifier(identifier),
        } = next_token
        else {
            return Err(ParseError {
                token: next_token,
                previous_span: Some(define_span),
                message: "`define` keyword must be followed by an identifier".to_string(),
            });
        };

        let next_token = self.advance();
        let prev_span = define_span.union(&ident_span);

        match next_token.value {
            T::NumericLiteral(number) => Ok(WithSpan {
                span: prev_span.union(&next_token.span),
                value: Stmt::AliasDefinition(identifier, AliasableThing::RawData(number)),
            }),
            T::GeneralRegisterName(reg) => Ok(WithSpan {
                span: prev_span.union(&next_token.span),
                value: Stmt::AliasDefinition(identifier, AliasableThing::Register(reg)),
            }),
            _ => Err(ParseError {
                token: next_token,
                previous_span: Some(prev_span),
                message: "Can only create aliases for raw data or general registers".to_string(),
            }),
        }
    }

    fn parse_raw_data_definition(&mut self) -> ParseResult<'s, SpanStmt<'s>> {
        let WithSpan {
            span: decl_span,
            value: decl,
        } = self.advance();
        let mut full_span = decl_span;
        let mut bytes = Vec::new();

        match decl {
            T::DefineBytes => {
                while let Some(&WithSpan { span: byte_span, value: T::NumericLiteral(byte) }) = self.peek() {
                    let byte_token = self.advance();
                    if byte > 255 {
                        return Err(ParseError { token: byte_token, previous_span: None, message: "Number in byte definition must only be 8 bit".to_string() });
                    }
                    full_span.mut_union(&byte_span);
                    bytes.push(byte as u8);
                }
            }
            T::DefineWords => {
                while let Some(&WithSpan { span: word_span, value: T::NumericLiteral(word) }) = self.peek() {
                    self.advance();
                    full_span.mut_union(&word_span);
                    bytes.extend(word.to_be_bytes());
                }
            }
            T::Text => {
                let token = self.advance();
                let WithSpan { span, value: T::StringLiteral(text) } = token else {
                    return Err(ParseError { token, previous_span: Some(decl_span), message: "Expected string literal after text data definition".to_string() });
                };
                full_span.mut_union(&span);
                bytes.extend(text.as_bytes());
            },
            _ => panic!("We should only call parse_raw_data_definition() when the previous token is a raw data definition")
        };

        Ok(WithSpan {
            span: full_span,
            value: Stmt::RawDataDefinition(bytes),
        })
    }

    /// include → "include" STRING_LITERAL;
    fn parse_include(&mut self) -> ParseResult<'s, SpanStmt<'s>> {
        let WithSpan {
            span: include_span,
            value: T::Include,
        } = self.advance()
        else {
            panic!("We should only call parse_include() when the previous token is Include");
        };

        let next_token = self.advance();
        let WithSpan {
            span: string_span,
            value: T::StringLiteral(filename),
        } = next_token
        else {
            return Err(ParseError {
                token: next_token,
                previous_span: Some(include_span),
                message: "`include` must be followed with a string literal".to_string(),
            });
        };

        Ok(WithSpan {
            span: include_span.union(&string_span),
            value: Stmt::Include(filename),
        })
    }

    /// label → IDENTIFIER ":";
    fn parse_label(&mut self) -> ParseResult<'s, SpanStmt<'s>> {
        let WithSpan {
            span: ident_span,
            value: T::Identifier(identifier),
        } = self.advance()
        else {
            panic!("We should only call parse_label() when the previous token is an identifier");
        };

        let next_token = self.advance();
        let WithSpan {
            span: colon_span,
            value: T::Colon,
        } = next_token
        else {
            return Err(ParseError {
                token: next_token,
                previous_span: Some(ident_span),
                message: "Label must be followed by `:`".to_string(),
            });
        };

        Ok(WithSpan {
            span: ident_span.union(&colon_span),
            value: Stmt::Label(identifier),
        })
    }
}
