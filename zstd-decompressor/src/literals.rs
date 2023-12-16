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
        let first = input.slice(1)?;
        let mut parser = ForwardBitParser::new(first).unwrap();

        let literal_type = parser.take(2).unwrap();

        todo!()
    }
}
