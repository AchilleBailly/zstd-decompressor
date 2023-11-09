extern crate zstd_decompressor;

use clap::{arg, command, Parser};
use color_eyre::{self, eyre};
use zstd_decompressor::{frame::Frame, parsing::ForwardByteParser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///ZStandard file input, decompress it and output to stdout
    #[arg(required = true)]
    filename: String,

    ///Dump information about frames instead of outputing the result
    #[arg(short, long)]
    info: bool,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let file = std::fs::read(args.filename)?;
    let parser = ForwardByteParser::new(file.as_slice());

    if args.info {
        for frame in parser.iter() {
            println!("{:#x?}", frame?);
        }
        return Ok(());
    }

    for frame in parser.iter() {
        match frame {
            Ok(Frame::SkippableFrame(_v)) => continue,
            Ok(Frame::ZStandardFrame()) => todo!(),
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}
