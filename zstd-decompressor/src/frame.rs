use std::path::Iter;

use crate::parsing::ForwardByteParser;

use eyre;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Unrecognised magic: {0}"}]
    UnrecognizedMagic(u32),
    #[error{"Parsing error: {0}"}]
    ParsingError(#[from] crate::parsing::Error),
}

const MAGIC_ZSTD: u32 = 0xFD2FB528;
const MAGIC_SKIP: u32 = 0x184D2A50; //

pub enum Frame<'a> {
    ZStandardFrame(),
    SkippableFrame(SkippableFrame<'a>),
}

pub struct SkippableFrame<'a> {
    pub magic: u32,
    pub data: &'a [u8],
}

type Result<T> = eyre::Result<T, Error>;

impl<'a> Frame<'a> {
    pub fn parse(input: &mut ForwardByteParser<'a>) -> Result<Self> {
        let magic = input.le_u32()?;

        match magic {
            MAGIC_ZSTD => todo!(),
            v if v ^ MAGIC_SKIP <= 0x0F => {
                let data_len = input.le_u32()? as usize;
                let sf = SkippableFrame {
                    magic: magic,
                    data: input.slice(data_len)?,
                };

                Ok(Frame::SkippableFrame(sf))
            }
            _ => Err(Error::UnrecognizedMagic(magic)),
        }
    }

    pub fn decode(self) -> Result<Vec<u8>> {
        match self {
            Self::SkippableFrame(frame) => Ok(frame.data.into()),
            Self::ZStandardFrame() => todo!(),
        }
    }
}

pub struct FrameIterator<'a> {
    parser: ForwardByteParser<'a>,
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = Result<Frame<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.len() {
            0 => None,
            _ => Some(Frame::parse(&mut self.parser)),
        }
    }
}
