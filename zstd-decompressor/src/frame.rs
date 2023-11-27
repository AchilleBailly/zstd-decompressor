use crate::{
    parsing::ForwardByteParser,
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
    #[error{"Dictionnary ID {0} is reserved but not registered"}]
    UnregisteredReservedDictID(u64),
}

const MAGIC_ZSTD: u32 = 0xFD2FB528;
const MAGIC_SKIP: u32 = 0x184D2A50; //

#[derive(Debug)]
pub enum Frame<'a> {
    ZStandardFrame(),
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

pub struct FrameHeader {
    content_checksum_flag: bool,
    window_size: Option<u64>,
    dictionnary_id: Option<u64>,
    content_size: Option<u64>,
}

impl FrameHeader {
    pub fn parse<'a>(input: &mut ForwardByteParser<'a>) -> Result<Self> {
        let header_descriptor = input.u8()?;

        let (header_descriptor, dict_id_flag) = get_n_bits(header_descriptor, 2);
        let (header_descriptor, content_checksum_flag) = get_n_bits(header_descriptor, 1);

        let (mut header_descriptor, reserved) = get_n_bits(header_descriptor, 1);
        if reserved != 0 {
            return Err(Error::ReservedSet("block header descriptor".to_string()));
            // See https://datatracker.ietf.org/doc/html/rfc8878#name-frame-header
        }

        // Unused bit, see https://datatracker.ietf.org/doc/html/rfc8878#section-3.1.1.1.1.3
        header_descriptor >>= 1;

        let (header_descriptor, single_segment_flag) = get_n_bits(header_descriptor, 1);
        let (_, content_size_flag) = get_n_bits(header_descriptor, 2);
        let fcs_field_size;
        if content_size_flag == 0 && single_segment_flag == 0 {
            fcs_field_size = None;
        } else if single_segment_flag == 1 {
            fcs_field_size = Some(1u8);
        } else {
            fcs_field_size = Some(2 ^ content_size_flag);
        }

        let window_size = if single_segment_flag == 1 {
            None
        } else {
            Some(Self::parse_window_descriptor(input)?)
        };

        let dict_id: Option<u64> = if dict_id_flag != 0 {
            let a = input.slice(2 ^ (dict_id_flag - 1) as usize)?;
            Some(int_from_array(a))
        } else {
            None
        };
        if let Some(v) = dict_id {
            if v <= 32767 || v >= (1 << 31) {
                return Err(Error::UnregisteredReservedDictID(v));
            }
        }

        let content_size = match fcs_field_size {
            None => None,
            Some(v) if v == 2 => Some(int_from_array::<u64>(input.slice(v as usize)?) + 256),
            Some(v) => Some(int_from_array(input.slice(v as usize)?)),
        };

        let window_size = if window_size == None {
            content_size
        } else {
            window_size
        };

        Ok(FrameHeader {
            content_checksum_flag: content_checksum_flag != 0,
            window_size: window_size,
            dictionnary_id: dict_id,
            content_size: content_size,
        })
    }

    fn parse_window_descriptor<'a>(input: &mut ForwardByteParser<'a>) -> Result<u64> {
        let window_descriptor = input.u8()?;
        let (exponent, mantissa) = get_n_bits(window_descriptor, 3);

        let window_base = 1 << (exponent as u64 + 10);
        let window_add = (window_base / 8) * mantissa as u64;

        Ok(window_base + window_add)
    }
}
