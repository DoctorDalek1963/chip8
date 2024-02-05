//! This module handles scanning the source code to tokenise it.

use crate::{
    span::{Span, WithSpan},
    tokens::{GeneralRegisterName, InstructionName, SpecialRegisterName, Token},
};

/// A scanner to tokenise the source code.
pub struct Scanner<'s> {
    /// The source code.
    source: &'s str,

    /// The tokens that we've already scanned out.
    tokens: Vec<WithSpan<Token<'s>>>,

    /// An index to the start of the token currently being scanned.
    start: usize,

    /// An index to the character currently being considered.
    current: usize,
}

impl<'s> Scanner<'s> {
    /// Scan all the tokens from the given source code.
    pub fn scan_tokens(source: &'s str) -> Vec<WithSpan<Token>> {
        let mut scanner = Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
        };

        while !scanner.is_at_end() {
            scanner.start = scanner.current;
            scanner.scan_token();
        }

        scanner.tokens
    }

    /// Are we at the end of the source code?
    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    /// Get the span from the start of this lexeme to the character most recently consumed.
    #[inline]
    fn current_span(&self) -> Span {
        Span {
            start: self.start,
            end: self.current - 1,
        }
    }

    /// Return the char pointed to by `self.current`.
    #[inline]
    fn current_char(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }

    /// Advance the internal pointer.
    fn advance(&mut self) -> char {
        let c = self.current_char().unwrap_or_else(|| {
            panic!(
                "source: {:?}, current: {}, tokens: {:?}",
                self.source, self.current, self.tokens
            )
        });
        self.current += 1;
        c
    }

    /// Add a token with the given token type and literal to the internal token vec.
    fn add_token(&mut self, token: Token<'s>) {
        self.tokens.push(WithSpan {
            span: self.current_span(),
            value: token,
        });
    }

    /// Report the given error message with the current span.
    fn report_error(&self, message: &str) {
        crate::error::report_error(self.current_span(), message);
    }

    /// Scan a single token.
    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            ';' => {
                while self.current_char() != Some('\n') && !self.is_at_end() {
                    self.advance();
                }
            }
            ':' => self.add_token(Token::Colon),
            ',' => {} // Ignore commas
            '"' => self.scan_string(),
            '0'..='9' => self.scan_decimal_number(),
            '%' => self.scan_binary_number(),
            '#' => self.scan_hex_number(),
            c if c.is_whitespace() => {}
            c if c.is_ascii_alphabetic() || c == '_' => self.scan_identifier_or_keyword(),
            _ => self.report_error(&format!("Unrecognised character: {c:?}")),
        }
    }

    /// Scan a string literal.
    fn scan_string(&mut self) {
        while self.current_char() != Some('"') && !self.is_at_end() {
            self.advance();
        }

        if self.is_at_end() {
            self.report_error("Unterminated string literal");
            return;
        }

        // The closing "
        self.advance();

        // Trim the surrounding quotes
        self.add_token(Token::StringLiteral(
            &self.source[(self.start + 1)..(self.current - 1)],
        ));
    }

    /// Scan a base 10 numeric literal.
    fn scan_decimal_number(&mut self) {
        while self.current_char().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        let num = match self.source[self.start..self.current].parse() {
            Ok(num) => num,
            Err(_) => {
                self.report_error("Numeric literal larger than 16 bits");
                return;
            }
        };

        self.add_token(Token::NumericLiteral(num));
    }

    /// Scan a binary numeric literal.
    fn scan_binary_number(&mut self) {
        // Ignore the leading %
        self.advance();

        while self.current_char().is_some_and(|c| c == '0' || c == '1') {
            self.advance();
        }

        let num = match u16::from_str_radix(&self.source[(self.start + 1)..self.current], 2) {
            Ok(num) => num,
            Err(_) => {
                self.report_error("Numeric literal larger than 16 bits");
                return;
            }
        };

        self.add_token(Token::NumericLiteral(num));
    }

    /// Scan a base 10 numeric literal.
    fn scan_hex_number(&mut self) {
        // Ignore the leading #
        self.advance();

        while self
            .current_char()
            .is_some_and(|c| ('0'..='9').contains(&c) || ('a'..='f').contains(&c))
        {
            self.advance();
        }

        let num = match u16::from_str_radix(&self.source[(self.start + 1)..self.current], 16) {
            Ok(num) => num,
            Err(_) => {
                self.report_error("Numeric literal larger than 16 bits");
                return;
            }
        };

        self.add_token(Token::NumericLiteral(num));
    }

    /// Scan a single identifier or keyword.
    fn scan_identifier_or_keyword(&mut self) {
        use GeneralRegisterName as G;
        use InstructionName as I;
        use SpecialRegisterName as S;

        /// Check if the given character is valid to be used in an identifier.
        fn is_ident_char(c: Option<char>) -> bool {
            c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        }

        while is_ident_char(self.current_char()) {
            self.advance();
        }

        let word_slice = &self.source[self.start..self.current];

        let token = match word_slice {
            // Instructions
            "nop" => Token::InstructionName(I::Nop),
            "cls" => Token::InstructionName(I::Cls),
            "ret" => Token::InstructionName(I::Ret),
            "jmp" | "jp" => Token::InstructionName(I::Jmp),
            "call" => Token::InstructionName(I::Call),
            "se" => Token::InstructionName(I::Se),
            "sne" => Token::InstructionName(I::Sne),
            "ld" => Token::InstructionName(I::Ld),
            "add" => Token::InstructionName(I::Add),
            "or" => Token::InstructionName(I::Or),
            "and" => Token::InstructionName(I::And),
            "xor" => Token::InstructionName(I::Xor),
            "sub" => Token::InstructionName(I::Sub),
            "subn" => Token::InstructionName(I::Subn),
            "shr" => Token::InstructionName(I::Shr),
            "shl" => Token::InstructionName(I::Shl),
            "rnd" => Token::InstructionName(I::Rnd),
            "drw" => Token::InstructionName(I::Drw),
            "skp" => Token::InstructionName(I::Skp),
            "sknp" => Token::InstructionName(I::Sknp),
            "delay" => Token::InstructionName(I::Delay),
            "sound" => Token::InstructionName(I::Sound),
            "font" | "hex" => Token::InstructionName(I::Font),
            "bcd" => Token::InstructionName(I::Bcd),
            "stor" => Token::InstructionName(I::Stor),
            "rstr" => Token::InstructionName(I::Rstr),

            // General registers
            "v0" => Token::GeneralRegisterName(G::V0),
            "v1" => Token::GeneralRegisterName(G::V1),
            "v2" => Token::GeneralRegisterName(G::V2),
            "v3" => Token::GeneralRegisterName(G::V3),
            "v4" => Token::GeneralRegisterName(G::V4),
            "v5" => Token::GeneralRegisterName(G::V5),
            "v6" => Token::GeneralRegisterName(G::V6),
            "v7" => Token::GeneralRegisterName(G::V7),
            "v8" => Token::GeneralRegisterName(G::V8),
            "v9" => Token::GeneralRegisterName(G::V9),
            "va" => Token::GeneralRegisterName(G::Va),
            "vb" => Token::GeneralRegisterName(G::Vb),
            "vc" => Token::GeneralRegisterName(G::Vc),
            "vd" => Token::GeneralRegisterName(G::Vd),
            "ve" => Token::GeneralRegisterName(G::Ve),
            "vf" => Token::GeneralRegisterName(G::Vf),

            // Special registers
            "i" => Token::SpecialRegisterName(S::I),
            "dt" => Token::SpecialRegisterName(S::Dt),
            "k" => Token::SpecialRegisterName(S::K),

            // Defines
            "define" => Token::Define,
            "db" => Token::DefineBytes,
            "dw" => Token::DefineWords,
            "text" => Token::Text,

            // Include
            "include" => Token::Include,

            // Identifier
            _ => Token::Identifier(word_slice),
        };

        self.add_token(token);
    }
}
