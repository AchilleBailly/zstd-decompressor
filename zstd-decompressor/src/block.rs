use crate::{decoding_context::DecodingContext, parsing::ForwardByteParser, utils::int_from_array};

use eyre;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Block is reserved"}]
    ReservedBlockType(),
    #[error{"Parsing error: {0}"}]
    ParsingError(#[from] crate::parsing::Error),
    #[error{"Block decoded size exceeds maximum accepted size."}]
    LargeBlockSize,
}

type Result<T> = eyre::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum Block<'a> {
    RawBlock(&'a [u8]),
    RLEBlock { byte: u8, repeat: u32 },
}

impl<'a> Block<'a> {
    pub fn parse(parser: &mut ForwardByteParser<'a>) -> Result<(Block<'a>, bool)> {
        let header = parser.slice(3)?;
        let mut header: u32 = int_from_array(header);

        let last_block = (header & 1) != 0;
        header >>= 1;

        let block_type = header & 0b11;
        header >>= 2;

        let block_size = header as usize;

        match block_type {
            // RawBlock
            0 => Ok((Block::RawBlock(parser.slice(block_size)?), last_block)),
            1 => Ok((
                Block::RLEBlock {
                    byte: parser.u8()?,
                    repeat: block_size as u32,
                },
                last_block,
            )),
            2 => unimplemented!(),
            _ => Err(Error::ReservedBlockType()),
        }
    }

    pub fn decode(self, context: &mut DecodingContext) -> Result<()> {
        let mut decoded = match self {
            Self::RawBlock(a) => Vec::from(a),
            Self::RLEBlock { byte, repeat } => vec![byte; repeat as usize],
        };

        if decoded.len() as u64 > context.window_size {
            return Err(Error::LargeBlockSize);
        } else if decoded.len() + context.decoded.len() > context.window_size as usize {
            for _ in 0..decoded.len() + context.decoded.len() - context.window_size as usize {
                context.decoded.remove(0);
            }
        }

        context.decoded.append(&mut decoded);

        Ok(())
    }
}
