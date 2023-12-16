use std::u8;

use bitbuffer::{BitError, BitReadBuffer, LittleEndian, BigEndian};
use eyre;
use thiserror;

use crate::{frame, utils::int_from_array};

pub struct ForwardByteParser<'a>(&'a [u8]);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Not enough bytes: {requested} requested out of {available} available."}]
    NotEnoughBytes { requested: usize, available: usize },
    #[error{"Not enough bits: {requested} requested out of {available} available."}]
    NotEnoughBits { requested: usize, available: usize },
    #[error{"Maximum readable bits (64) exceded: requested {0}."}]
    MaximumReadableBitsExceeded(usize),
    #[error{"Given data is empty."}]
    EmptyInputData,
    #[error{"The first byte is null"}]
    NullByte,
    #[error{"not the good number of bits available in the buffer"}]
    NumberBitsError(#[from] BitError),
}

pub type Result<T, E = Error> = eyre::Result<T, E>;

impl<'a> ForwardByteParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self(data)
    }

    pub fn iter(self) -> frame::FrameIterator<'a> {
        return frame::FrameIterator { parser: self };
    }

    /// Retrieve the next byte unparsed
    pub fn u8(&mut self) -> Result<u8> {
        match self.0.split_first() {
            Some((first, rest)) => {
                self.0 = rest;
                Ok(*first)
            }
            None => Err(Error::NotEnoughBytes {
                requested: 1,
                available: 0,
            }),
        }
    }

    /// Return the number of bytes still unparsed
    pub fn len(&self) -> usize {
        return self.0.len();
    }

    /// Check if the input is exhausted
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Extract `len` bytes as a slice
    pub fn slice(&mut self, len: usize) -> Result<&'a [u8]> {
        let old_len = self.len();
        if old_len < len {
            // Case where there are fewer bytes available than len
            return Err(Error::NotEnoughBytes {
                requested: len,
                available: old_len,
            });
        }

        let ret = &self.0[..len];
        self.0 = &self.0[len..];
        Ok(ret)
    }

    /// Consume and returns a u32 in little-endian format
    pub fn le_u32(&mut self) -> Result<u32> {
        if self.len() < 4 {
            return Err(Error::NotEnoughBytes {
                requested: 4,
                available: self.len(),
            });
        }

        let (res_array, rest) = self.0.split_at(4);
        self.0 = rest;
        let res: u32 = int_from_array(res_array);

        Ok(res)
    }
}

pub trait BitParser<'a> {
    /// Create a new BitParser from the data
    fn new(data: &'a mut [u8]) -> Result<Self>
    where
        Self: Sized;

    /// Return the number of readable bits left in the bitparser
    fn len(&self) -> usize;

    /// Tell if the BitParser has any bit left to be read
    fn is_empty(&self) -> bool;

    /// Take len bits from the BitParse, consuming them
    fn take(&mut self, len: usize) -> Result<u64>;

    /// Return the value from the len next bits without consuming them
    fn peek(&self, len: usize) -> Result<u64>;
}

pub struct ForwardBitParser<'a> {
    data: BitReadBuffer<'a, LittleEndian>,
    readable: usize,
    pos: usize,
}

impl<'a> BitParser<'a> for ForwardBitParser<'a> {
    /// Create a new forward bit parser
    fn new(data: &'a mut [u8]) -> Result<Self> {
        if data.len() == 0 {
            return Err(Error::EmptyInputData);
        }
        Ok(ForwardBitParser {
            data: BitReadBuffer::new(data, LittleEndian),
            readable: data.len() * 8,
            pos: 0,
        })
    }

    fn len(&self) -> usize {
        self.readable
    }

    /// True if there are no bits left to read
    fn is_empty(&self) -> bool {
        self.readable == 0
    }

    /// Get the given number of bits, or return an error.
    fn take(&mut self, len: usize) -> Result<u64> {
        if self.data.bit_len() < len {
            return Err(Error::NotEnoughBits {
                requested: len,
                available: self.len(),
            });
        }
        if len > 64 {
            return Err(Error::MaximumReadableBitsExceeded(len));
        }

        // we verified up there the conditions, can not fail
        let res = self.data.read_int(self.pos, len).unwrap();

        self.readable -= len;
        self.pos += len;

        Ok(res)
    }

    /// Peek at next len bits without consuming them
    fn peek(&self, len: usize) -> Result<u64> {
        if self.data.bit_len() < len {
            return Err(Error::NotEnoughBits {
                requested: len,
                available: self.data.bit_len(),
            });
        }
        if len > 64 {
            return Err(Error::MaximumReadableBitsExceeded(len));
        }

        // we verified up there the conditions, can not fail
        let res = self.data.read_int(self.pos, len).unwrap();

        Ok(res)
    }
}

pub struct BackwardBitParser<'a> {
    bytes: BitReadBuffer<'a, BigEndian>,
    readable:usize,
    pos: usize,
    
}

impl<'a> BitParser<'a> for BackwardBitParser<'a> {

    /// Create a new backward bit parser. The header is skipped automatically or
    /// an error is returned if the initial 1 cannot be found in the first 8 bits.
    fn new(data: &'a mut [u8]) -> Result<Self> {
        if data.len() == 0 {
            return Err(Error::EmptyInputData);
        }
        let mut i = 0;
        if data[data.len() - 1] == 1 {
            data.reverse();
            return Ok(BackwardBitParser {
                bytes: BitReadBuffer::new(data, BigEndian),
                readable: data.len() * 8 - 8,
                pos: 8,
            });
        } else {
            while data[data.len() - 1] & 1 << 7 - i == 0 {
                    i+=1;
                if i == 7 {
                    return Err(Error::NullByte);
                }
            }
    
        

            data.reverse();
            Ok(BackwardBitParser {
                bytes: BitReadBuffer::new(data, BigEndian),
                readable: data.len() * 8 - i + 1,
                pos: i+1,
            })
        }
    }



    /// True if there are no bits left to read
    fn is_empty(&self) -> bool {
        self.readable == 0
    }

    /// Get the given number of bits, or return an error.
    fn take(&mut self, len: usize) -> Result<u64> {
        if self.bytes.bit_len() < len {
            return Err(Error::NotEnoughBits {
                requested: len,
                available: self.len(),
            });
        }
        if len > 64 {
            return Err(Error::MaximumReadableBitsExceeded(len));
        }


        // we verified up there the conditions, can not fail

        let res = self.bytes.read_int(self.pos, len).unwrap();

        self.readable -= len;
        self.pos += len;

        Ok(res)
    }

   

    fn len(&self) -> usize {
        self.readable
    }

    fn peek(&self, _len: usize) -> Result<u64> {
        todo!()
    }
}
