use crate::{
    decoders::{self, huffman::HuffmanDecoder},
    decoding_context::DecodingContext,
    parsing::{self, BackwardBitParser, ForwardBitParser, ForwardByteParser},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Parsing Error: {0}"}]
    ParsingError(#[from] parsing::Error),
    #[error{"Error while building tree: {0}"}]
    HuffmanTree(#[from] decoders::Error),
    #[error{"No huffman decoder available"}]
    HuffmanDecoderMissing,
    #[error{"Corrupted literals section: sum of streams sizes is too big"}]
    CorruptedStreamsSizeTooBig,
}

type Result<T> = eyre::Result<T, Error>;

#[derive(Debug)]
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
        jump_table: [u16; 4],
        data: &'a [u8],
    },
}

enum LiteralType {
    Raw = 0,
    Rle = 1,
    Compressed = 2,
    Treeless = 3,
}

impl<'a> LiteralsSection<'a> {
    /// Decompress the literals section. Update the Huffman decoder in
    /// `context` if appropriate (compressed literals block with a
    /// Huffman table inside).
    pub fn decode(self, context: &mut DecodingContext) -> Result<Vec<u8>> {
        match self {
            LiteralsSection::RawLiteralsBlock { data } => Ok(data.to_owned()),
            LiteralsSection::RLELiteralsBlock { byte, repeat } => Ok(vec![byte; repeat as usize]),
            LiteralsSection::CompressedLiteralsBlock {
                huffman_decoder,
                regenerated_size: _,
                jump_table,
                data,
            } => {
                if huffman_decoder.is_some() {
                    context.huffman_decoder = huffman_decoder;
                }

                let decoder = match &context.huffman_decoder {
                    None => return Err(Error::HuffmanDecoderMissing),
                    Some(h) => h,
                };

                let mut res = vec![];
                let mut data = data;
                for stream_size in jump_table {
                    if stream_size == 0 {
                        break;
                    }

                    let mut parser = BackwardBitParser::new(&data[..stream_size as usize])?;
                    data = &data[stream_size as usize..];

                    while !parser.is_empty() {
                        res.push(decoder.decode(&mut parser)?);
                    }
                }

                Ok(res)
            }
        }
    }

    pub fn parse(input: &mut ForwardByteParser<'a>) -> Result<Self> {
        let (lit_type, regen_size, compressed_size, n_streams) = Self::parse_header(input)?;

        match lit_type {
            LiteralType::Raw => Ok(LiteralsSection::RawLiteralsBlock {
                data: input.slice(regen_size)?,
            }),
            LiteralType::Rle => Ok(LiteralsSection::RLELiteralsBlock {
                byte: input.u8()?,
                repeat: regen_size as u32,
            }),
            v => {
                let mut new_input = ForwardByteParser::new(input.slice(compressed_size)?);
                let tree = if matches!(v, LiteralType::Treeless) {
                    None
                } else {
                    Some(HuffmanDecoder::parse(&mut new_input)?)
                };

                let total_streams_size = new_input.len();
                let jump_table = if n_streams == 4 {
                    let (s1, s2, s3) = (
                        new_input.le_u16()?,
                        new_input.le_u16()?,
                        new_input.le_u16()?,
                    );

                    if s1 as usize + s2 as usize + s3 as usize > total_streams_size - 6 {
                        return Err(Error::CorruptedStreamsSizeTooBig);
                    }

                    let s4 = total_streams_size - 6 - s1 as usize - s2 as usize - s3 as usize;
                    [s1, s2, s3, s4 as u16]
                } else {
                    [new_input.len() as u16, 0, 0, 0]
                };

                Ok(LiteralsSection::CompressedLiteralsBlock {
                    huffman_decoder: tree,
                    regenerated_size: regen_size,
                    jump_table,
                    data: new_input.slice(new_input.len())?,
                })
            }
        }
    }

    fn parse_header(input: &mut ForwardByteParser<'a>) -> Result<(LiteralType, usize, usize, u8)> {
        let header = input.u8()?;
        let binding = [header];
        let mut parser = ForwardBitParser::new(&binding).unwrap();

        let literal_type = parser.take(2).unwrap();

        let (regen_size, compressed_size, n_streams) = match literal_type {
            0 | 1 => (
                // RLE or Raw literals type
                match parser.peek(2).unwrap() {
                    0 | 2 => header as usize >> 3,
                    1 => ((header as usize) >> 4) + ((input.u8()? as usize) << 4),
                    3 => {
                        ((header as usize) >> 4)
                            + ((input.u8()? as usize) << 4)
                            + ((input.u8()? as usize) << 12)
                    }
                    _ => unreachable!(),
                },
                0,
                1,
            ),
            2 | 3 => match parser.take(2).unwrap() {
                // Compressed or TreeLess
                0 => {
                    let mut parser = ForwardBitParser::new(input.slice(2)?).unwrap();
                    (
                        ((header as usize) >> 4) + parser.take(6).unwrap() as usize,
                        parser.take(10).unwrap() as usize,
                        1,
                    )
                }
                1 => {
                    let mut parser = ForwardBitParser::new(input.slice(2)?).unwrap();
                    (
                        ((header as usize) >> 4) + parser.take(6).unwrap() as usize,
                        parser.take(10).unwrap() as usize,
                        4,
                    )
                }
                2 => {
                    let mut parser = ForwardBitParser::new(input.slice(3)?).unwrap();
                    (
                        ((header as usize) >> 4) + parser.take(8).unwrap() as usize,
                        parser.take(14).unwrap() as usize,
                        4,
                    )
                }
                3 => {
                    let mut parser = ForwardBitParser::new(input.slice(4)?).unwrap();
                    (
                        ((header as usize) >> 4) + parser.take(14).unwrap() as usize,
                        parser.take(18).unwrap() as usize,
                        4,
                    )
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        };

        let lit_type = match literal_type {
            0 => LiteralType::Raw,
            1 => LiteralType::Rle,
            2 => LiteralType::Compressed,
            3 => LiteralType::Treeless,
            _ => unreachable!(),
        };

        Ok((lit_type, regen_size, compressed_size, n_streams))
    }
}
