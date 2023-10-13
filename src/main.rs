extern crate zstd_decompressor;
use color_eyre::{self, eyre};
use zstd_decompressor::parsing::ForwardByteParser;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let mut parser = ForwardByteParser::new(&[0x12, 0x23, 0x34]);
    assert_eq!(0x12, parser.u8().unwrap());
    assert_eq!(0x23, parser.u8().unwrap());
    assert_eq!(0x34, parser.u8().unwrap());
    parser.u8()?;
    Ok(())
}
