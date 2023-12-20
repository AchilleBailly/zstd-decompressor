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

    ///Output to given file (overwritting) instead of writing to stdout
    #[arg(short, long, value_names = ["filename"])]
    output: Option<String>,

    ///Output Skippable frames as well
    #[arg(short, long, action)]
    print_skippable: bool,
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

    let mut res: Vec<u8> = vec![];
    for frame in parser.iter() {
        match frame {
            Ok(Frame::SkippableFrame(skippable)) => {
                if args.print_skippable {
                    res.append(&mut Frame::SkippableFrame(skippable).decode()?)
                }
            }
            Ok(frame) => res.append(&mut frame.decode()?),
            Err(e) => return Err(e.into()),
        }
    }
    if args.output.is_some() {
        std::fs::write(args.output.unwrap(), String::from_utf8(res).unwrap())?;
    } else {
        print!("{}", String::from_utf8(res).unwrap());
    }
    Ok(())
}
