extern crate zstd_decompressor;

use clap::{arg, command, Parser};
use color_eyre::{self, eyre};
use zstd_decompressor::parsing::ForwardByteParser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///ZStandard file input
    #[arg(required = true)]
    filename: String,

    ///Dump information about frames instead of outputing the result
    #[arg(short, long)]
    info: bool,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    if args.info {
        let file = std::fs::read(args.filename)?;
        let parser = ForwardByteParser::new(file.as_slice());
        for frame in parser.iter() {
            println!("{:#x?}", frame?);
        }
    }

    Ok(())
}
