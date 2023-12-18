use crate::parsing::BackwardBitParser;

use super::{BitDecoder, Result};

#[derive(Clone)]
pub struct RLEDecoder {
    byte: u8,
}

impl RLEDecoder {
    pub fn new(byte: u8) -> Self {
        RLEDecoder { byte }
    }
}

impl<'a> BitDecoder<u16> for RLEDecoder {
    fn initialize(&mut self, _bitstream: &mut BackwardBitParser) -> Result<()> {
        Ok(())
    }

    fn expected_bits(&self) -> usize {
        0
    }

    fn symbol(&mut self) -> u16 {
        self.byte as u16
    }

    fn update_bits(&mut self, _bitstream: &mut BackwardBitParser) -> Result<bool> {
        Ok(false)
    }

    fn reset(&mut self) {}
}
