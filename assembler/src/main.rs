//! This is a simple assembler for a simple CHIP-8 assembly language. See the README for more
//! details.

use clap::Parser;
use color_eyre::Result;
use std::fs;

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

    Ok(())
}
