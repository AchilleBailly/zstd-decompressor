use thiserror;
use eyre;

pub struct ForwardByteParser<'a>(&'a [u8]);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Input Empty"}]
    InputEmpty(),
}

pub type Result<T, E=Error> = eyre::Result<T,E>;



impl<'a> ForwardByteParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self(data)
    }

    pub fn u8(&mut self) -> Result<u8> {
        match self.0.split_first() {
            Some((first, rest)) => {self.0 = rest;
                                                Ok(*first)}
            None => Err(Error::InputEmpty())
        }
    }
}




