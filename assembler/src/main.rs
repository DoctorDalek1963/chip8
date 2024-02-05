//! This is a simple assembler for a simple CHIP-8 assembly language. See the README for more
//! details.

mod ast;
mod error;
mod parser;
mod scanner;
mod span;
mod tokens;

use crate::{
    error::{init_error_reporting, HAD_ERROR},
    parser::Parser,
    scanner::Scanner,
};
use color_eyre::{Report, Result};
use std::{fs, sync::atomic::Ordering};

#[derive(clap::Parser)]
#[command(author, version, about)]
struct Args {
    /// The filename of the code to assemble.
    file: String,

    /// The name of the file to output the assembled ROM to.
    #[arg(long, short)]
    output: String,
}

fn main() -> Result<()> {
    let args = <Args as clap::Parser>::parse();

    let input = fs::read_to_string(args.file)?.replace("\t", "    ");
    init_error_reporting(input.clone());
    let lowercase_input = input.to_ascii_lowercase();

    let tokens = Scanner::scan_tokens(&lowercase_input);

    if HAD_ERROR.load(Ordering::Relaxed) {
        return Err(Report::msg("Failed to tokenise input"));
    }

    dbg!(tokens
        .iter()
        .map(|crate::span::WithSpan { span: _, value }| value)
        .collect::<Vec<_>>());

    let unresolved_ast = Parser::parse(tokens);

    dbg!(unresolved_ast);

    Ok(())
}
