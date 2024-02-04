//! This is a simple assembler for a simple CHIP-8 assembly language. See the README for more
//! details.

mod error;
mod scanner;
mod span;
mod tokens;

use self::scanner::Scanner;
use crate::error::{init_error_reporting, HAD_ERROR};
use clap::Parser;
use color_eyre::{Report, Result};
use std::{fs, sync::atomic::Ordering};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// The filename of the code to assemble.
    file: String,

    /// The name of the file to output the assembled ROM to.
    #[arg(long, short)]
    output: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = fs::read_to_string(args.file)?;
    init_error_reporting(input.clone());
    let lowercase_input = input.to_ascii_lowercase();

    let tokens = Scanner::scan_tokens(&lowercase_input);

    if HAD_ERROR.load(Ordering::Relaxed) {
        return Err(Report::msg("Failed to tokenise input"));
    }

    dbg!(tokens
        .into_iter()
        .map(|crate::span::WithSpan { span: _, value }| value)
        .collect::<Vec<_>>());

    Ok(())
}
