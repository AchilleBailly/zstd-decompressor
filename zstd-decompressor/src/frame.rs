use std::{any::type_name, hash::Hasher};

use crate::{
    block::Block,
    decoding_context::{self, DecodingContext},
    parsing::{self, ForwardBitParser, ForwardByteParser},
    utils::{get_n_bits, int_from_array},
};

use eyre;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Unrecognised magic: {0}"}]
    UnrecognizedMagic(u32),
    #[error{"Parsing error: {0}"}]
    ParsingError(#[from] crate::parsing::Error),
    #[error{"Reserved value in {0} was set"}]
    ReservedSet(String),
    #[error{"Unvalid checksum in {0}"}]
    UnvalidChecksum(String),
    #[error{"Block error: {0}"}]
    BlockError(#[from] crate::block::Error),
    #[error{"Dictionnary ID {0} is reserved but not registered"}]
    UnregisteredReservedDictID(u64),
    #[error{"Expected checksum from header but is not present"}]
    MissingChecksum(#[source] parsing::Error),
    #[error{"Bad checksum, data was lost or modified"}]
    BadCheksum,
    #[error{"Window size is too big: max {max} but got {got}"}]
    WindowSizeTooBig { max: u64, got: u64 },
    #[error{"Decoded block's size exceeded the annonced content size: "}]
    ContentSizeTooBig(),
    #[error{"Bad Offset value (0)"}]
    NullOffsetError,
    #[error{"Decoding context error: {0}"}]
    DecodingContextError(#[from] decoding_context::Error),
}

const MAGIC_ZSTD: u32 = 0xFD2FB528;
const MAGIC_SKIP: u32 = 0x184D2A50; //

pub const MAX_WIN_SIZE: u64 = 8 << 20; // 8MiB

#[derive(Debug)]
pub enum Frame<'a> {
    ZStandardFrame(ZStandardFrame<'a>),
    SkippableFrame(SkippableFrame<'a>),
}

#[derive(Debug)]
pub struct SkippableFrame<'a> {
    pub magic: u32,
    pub data: &'a [u8],
}

type Result<T> = eyre::Result<T, Error>;

impl<'a> Frame<'a> {
    pub fn parse(input: &mut ForwardByteParser<'a>) -> Result<Self> {
        let magic = input.le_u32()?;

        match magic {
            MAGIC_ZSTD => Ok(Frame::ZStandardFrame(ZStandardFrame::parse(input)?)),
            v if v ^ MAGIC_SKIP <= 0x0F => {
                let data_len = input.le_u32()? as usize;
                let sf = SkippableFrame {
                    magic,
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
            Self::ZStandardFrame(frame) => frame.decode(),
        }
    }
}

pub struct FrameIterator<'a> {
    pub parser: ForwardByteParser<'a>,
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

#[derive(Debug)]
pub struct FrameHeader {
    pub content_checksum_flag: bool,
    pub window_size: u64,
    pub dictionnary_id: Option<u64>,
    pub content_size: Option<u64>,
}

impl FrameHeader {
    pub fn parse(input: &mut ForwardByteParser<'_>) -> Result<Self> {
        let mut header = ForwardBitParser::new(input.slice(1)?).unwrap();

        let dict_id_flag = header.take(2).unwrap();
        let content_checksum_flag = header.take(1).unwrap();

        let reserved = header.take(1).unwrap();
        if reserved != 0 {
            return Err(Error::ReservedSet(type_name::<Self>().to_string()));
            // See https://datatracker.ietf.org/doc/html/rfc8878#name-frame-header
        }

        // Unused bit, see https://datatracker.ietf.org/doc/html/rfc8878#section-3.1.1.1.1.3
        header.take(1).unwrap();

        let single_segment_flag = header.take(1).unwrap();
        let content_size_flag = header.take(2).unwrap();
        let fcs_field_size;
        if content_size_flag == 0 && single_segment_flag == 0 {
            fcs_field_size = None;
        } else if content_size_flag == 0 && single_segment_flag == 1 {
            fcs_field_size = Some(1u8);
        } else {
            fcs_field_size = Some(1 << content_size_flag);
        }

        let window_size = if single_segment_flag == 1 {
            None
        } else {
            Some(Self::parse_window_descriptor(input)?)
        };

        let dict_id: Option<u64> = if dict_id_flag != 0 {
            let a = input.slice(1 << (dict_id_flag - 1) as usize)?;
            Some(int_from_array(a))
        } else {
            None
        };
        // if let Some(v) = dict_id {
        //     if v <= 32767 || v >= (1 << 31) {
        //         return Err(Error::UnregisteredReservedDictID(v)); // TODO: check registration of the dict ID
        //     }
        // }

        let content_size = match fcs_field_size {
            None => None,
            Some(v) => {
                let mut bit_parser = ForwardBitParser::new(input.slice(v as usize)?)?;
                let mut res = bit_parser.take(v as usize * 8).unwrap();
                if v == 2 {
                    res += 256;
                }
                Some(res)
            }
        };

        let window_size = window_size
            .or(content_size)
            .expect("Invalid configuration, no window size and no content_size !");

        Ok(FrameHeader {
            content_checksum_flag: content_checksum_flag != 0,
            window_size,
            dictionnary_id: dict_id,
            content_size,
        })
    }

    fn parse_window_descriptor(input: &mut ForwardByteParser<'_>) -> Result<u64> {
        let window_descriptor = input.u8()?;
        let (mantissa, exponent) = get_n_bits(window_descriptor, 3);

        let window_base = 1 << (exponent as u64 + 10);
        let window_add = (window_base / 8) * mantissa as u64;

        Ok(window_base + window_add)
    }
}

#[derive(Debug)]
pub struct ZStandardFrame<'a> {
    header: FrameHeader,
    blocks: Vec<Block<'a>>,
    checksum: Option<u32>,
}

impl<'a> ZStandardFrame<'a> {
    pub fn parse(input: &mut ForwardByteParser<'a>) -> Result<Self> {
        let header = FrameHeader::parse(input)?;

        if header.window_size > MAX_WIN_SIZE {
            return Err(Error::WindowSizeTooBig {
                max: MAX_WIN_SIZE,
                got: header.window_size,
            });
        }

        let mut blocks: Vec<Block> = vec![];

        loop {
            let (cur, last) = Block::parse(input)?;
            blocks.push(cur);

            if last {
                break;
            }
        }

        let checksum = if header.content_checksum_flag {
            Some(input.le_u32().map_err(Error::MissingChecksum)?)
        } else {
            None
        };

        Ok(ZStandardFrame {
            header,
            blocks,
            checksum,
        })
    }

    pub fn decode(self) -> Result<Vec<u8>> {
        let mut context: DecodingContext = DecodingContext::new(self.header.window_size)?;

        for block in self.blocks.into_iter() {
            block.decode(&mut context)?; // Copying block content, TODO: check if possible other way
            if let Some(mut hash) = context.checksum { hash.write(&context.decoded) }
        }

        if self.header.content_checksum_flag {
            context.checksum = Some(twox_hash::XxHash64::with_seed(0));

            if context.checksum.unwrap().finish() as u32 == self.checksum.unwrap() {
            } else {
                println!("Warning: Bad checksum !");
            }

            Ok(context.decoded)
        } else {
            Ok(context.decoded)
        }
    }

    pub fn header(&self) -> &FrameHeader {
        &self.header
    }

    pub fn checksum(&self) -> Option<u32> {
        self.checksum
    }

    pub fn blocks(&self) -> &Vec<Block<'a>> {
        &self.blocks
    }
}

#[cfg(test)]
mod parse_window_descriptor_tests {
    use crate::parsing::ForwardByteParser;

    use super::FrameHeader;

    #[test]
    fn parse_window_descriptor_min_ok() {
        let mut parser = ForwardByteParser::new(&[0]);

        assert_eq!(
            1 << 10,
            FrameHeader::parse_window_descriptor(&mut parser).unwrap()
        );
    }

    #[test]
    fn parse_window_descriptor_max_ok() {
        let mut parser = ForwardByteParser::new(&[0xff]);

        assert_eq!(
            (1 << 41) + 7 * (1 << 38),
            FrameHeader::parse_window_descriptor(&mut parser).unwrap()
        );
    }

    #[test]
    fn parse_window_descriptor_min_and_1_ok() {
        let mut parser = ForwardByteParser::new(&[0b00000001]);

        assert_eq!(
            (1 << 10) + 1024 / 8,
            FrameHeader::parse_window_descriptor(&mut parser).unwrap()
        );
    }
}
