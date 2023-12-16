use std::time::UNIX_EPOCH;

use crate::{
    decoders::huffman::HuffmanDecoder,
    parsing::{self, BitParser, ForwardBitParser, ForwardByteParser},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Parsing Error: {0}"}]
    ParsingError(#[from] parsing::Error),
}

type Result<T> = eyre::Result<T, Error>;

pub enum LiteralsSection<'a> {
    RawLiteralsBlock {
        data: &'a [u8],
    },
    RLELiteralsBlock {
        byte: u8,
        repeat: u32,
    },
    CompressedLiteralsBlock {
        huffman_decoder: Option<HuffmanDecoder>,
        regenerated_size: usize,
        jump_table: [u8; 3],
        data: &'a [u8],
    },
}

enum LiteralType {
    Raw = 0,
    RLE = 1,
    Compressed = 2,
    Treeless = 3,
}

impl<'a> LiteralsSection<'a> {
    pub fn parse(input: &mut ForwardByteParser<'a>) -> Result<Self> {
        let (lit_type, regen_size, compressed_size, n_streams) = Self::parse_header(input)?;
        todo!()
    }

    fn parse_header(input: &mut ForwardByteParser<'a>) -> Result<(LiteralType, u32, u32, u8)> {
        let header = input.u8()?;
        let binding = [header];
        let mut parser = ForwardBitParser::new(&binding).unwrap();

        let literal_type = parser.take(2).unwrap();

        let (regen_size, compressed_size, n_streams) = match literal_type {
            0 | 1 => (
                // RLE or Raw literals type
                match parser.peek(2).unwrap() {
                    0 | 2 => header as u32 >> 3,
                    1 => ((header as u32) >> 4) + ((input.u8()? as u32) << 4),
                    3 => {
                        ((header as u32) >> 4)
                            + ((input.u8()? as u32) << 4)
                            + ((input.u8()? as u32) << 12)
                    }
                    _ => unreachable!(),
                },
                0,
                1,
            ),
            2 | 3 => match parser.take(2).unwrap() {
                // Compressed or TreeLess
                0 => {
                    let mut parser = ForwardBitParser::new(input.slice(3)?).unwrap();
                    (
                        ((header as u32) >> 4) + parser.take(6).unwrap() as u32,
                        parser.take(10).unwrap() as u32,
                        1,
                    )
                }
                1 => {
                    let mut parser = ForwardBitParser::new(input.slice(3)?).unwrap();
                    (
                        ((header as u32) >> 4) + parser.take(6).unwrap() as u32,
                        parser.take(10).unwrap() as u32,
                        4,
                    )
                }
                2 => {
                    let mut parser = ForwardBitParser::new(input.slice(4)?).unwrap();
                    (
                        ((header as u32) >> 4) + parser.take(8).unwrap() as u32,
                        parser.take(14).unwrap() as u32,
                        4,
                    )
                }
                3 => {
                    let mut parser = ForwardBitParser::new(input.slice(5)?).unwrap();
                    (
                        ((header as u32) >> 4) + parser.take(14).unwrap() as u32,
                        parser.take(18).unwrap() as u32,
                        4,
                    )
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        let lit_type = match literal_type {
            0 => LiteralType::Raw,
            1 => LiteralType::RLE,
            2 => LiteralType::Compressed,
            3 => LiteralType::Treeless,
            _ => unreachable!(),
        };

        Ok((lit_type, regen_size, compressed_size, n_streams))
    }
}
