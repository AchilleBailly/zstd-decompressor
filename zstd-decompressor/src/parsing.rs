use eyre;
use thiserror;

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
        todo!()
    }

    /// Check if the input is exhausted
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Extract `len` bytes as a slice
    pub fn slice(&mut self, len: usize) -> Result<&'a [u8]> {
        todo!()
    }

    /// Consume and return a u32 in little-endian format
    pub fn le_u32(&mut self) -> Result<u32> {
        todo!()
    }
}