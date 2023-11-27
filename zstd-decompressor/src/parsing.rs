use std::u8;

use eyre;
use thiserror;

use crate::{frame, utils::int_from_array};

pub struct ForwardByteParser<'a>(&'a [u8]);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Not enough byte: {requested} requested out of {available} available."}]
    NotEnoughBytes { requested: usize, available: usize },
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
