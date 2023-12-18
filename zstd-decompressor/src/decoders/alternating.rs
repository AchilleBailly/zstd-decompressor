use crate::parsing::BackwardBitParser;

use super::{
    fse::{FseDecoder, FseTable},
    BitDecoder, Result,
};

pub struct AlternatingDecoder {
    pub first_decoder: FseDecoder,
    pub second_decoder: FseDecoder,
    pub last_updated_is_first: bool,
    pub last_read_is_first: bool,
}

impl AlternatingDecoder {
    pub fn new(table: FseTable) -> Self {
        let bis_table = table.clone();
        AlternatingDecoder {
            first_decoder: FseDecoder::new_from_table(table),
            second_decoder: FseDecoder::new_from_table(bis_table),
            last_updated_is_first: false,
            last_read_is_first: false,
        }
    }
}

impl<'a> BitDecoder<u16> for AlternatingDecoder {
    fn initialize(&mut self, bitstream: &mut BackwardBitParser) -> Result<()> {
        self.first_decoder.initialize(bitstream)?;
        self.second_decoder.initialize(bitstream)?;
        self.last_updated_is_first = false;
        self.last_read_is_first = false;
        Ok(())
    }

    fn expected_bits(&self) -> usize {
        if self.last_updated_is_first {
            return self.second_decoder.expected_bits();
        } else {
            return self.first_decoder.expected_bits();
        }
    }

    fn symbol(&mut self) -> u16 {
        if self.last_read_is_first {
            self.last_read_is_first = false;
            self.second_decoder.symbol()
        } else {
            self.last_read_is_first = true;
            self.first_decoder.symbol()
        }
    }

    fn update_bits(&mut self, bitstream: &mut BackwardBitParser) -> Result<bool> {
        if self.last_updated_is_first {
            self.last_updated_is_first = false;
            Ok(self.second_decoder.update_bits(bitstream)?)
        } else {
            self.last_updated_is_first = true;
            Ok(self.first_decoder.update_bits(bitstream)?)
        }
    }

    fn reset(&mut self) {
        self.first_decoder.reset();
        self.second_decoder.reset();
        self.last_updated_is_first = false;
    }
}
