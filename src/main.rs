extern crate zstd_decompressor;
use std::string;

use color_eyre::{self, eyre};
use zstd_decompressor::parsing::ForwardByteParser;
use clap::{Parser, command, arg};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///Dump information about frames instead of outputing the result
    #[arg(short, long)]
    info: String,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    
    let file = std::fs::read(args.info);
    
    

    let mut parser = ForwardByteParser::new(&[0x12, 0x23, 0x34]);
    assert_eq!(0x12, parser.u8().unwrap());
    assert_eq!(0x23, parser.u8().unwrap());
    assert_eq!(0x34, parser.u8().unwrap());
    parser.u8()?;
    Ok(())
}
