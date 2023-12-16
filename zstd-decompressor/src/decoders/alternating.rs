

use super::{BitDecoder, fse::{FseDecoder, FseTable, self}};

use std::{
    fmt::{Debug, Display},
    
};


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Error {0} from the FseTable"}]
    FseError(#[from] fse::Error)
}



pub struct AlternatingDecoder {
    first_decoder : FseDecoder,
    second_decoder : FseDecoder,
    last_updated_is_first : bool
}


impl AlternatingDecoder {

    fn new(table : FseTable) -> Self {
        let bis_table = table.clone();
        AlternatingDecoder {
            first_decoder: FseDecoder::from(table),
            second_decoder: FseDecoder::from(bis_table),
            last_updated_is_first: false,
        }

    }

}


impl <'a>BitDecoder<'a, Error, u16> for AlternatingDecoder {
    fn initialize(&mut self, bitstream: &mut impl crate::parsing::BitParser<'a>) -> Result<(), Error> {
        self.first_decoder.initialize(bitstream);
        self.last_updated_is_first = true;
        self.second_decoder.initialize(bitstream);
        self.last_updated_is_first= false;
        Ok(())
    }

    fn expected_bits(&self) -> usize {
        if self.last_updated_is_first {
            return self.second_decoder.expected_bits();
        }
        else {
            return self.first_decoder.expected_bits();
        }
    }

    fn symbol(&mut self) -> u16 {
        if self.last_updated_is_first {
            self.last_updated_is_first = false;
            return self.second_decoder.symbol(); 
        }
        else {
            self.last_updated_is_first = true;
            return self.first_decoder.symbol();
        }
    }

    fn update_bits(&mut self, bitstream: &mut impl crate::parsing::BitParser<'a>) -> Result<bool, Error> {
        if self.last_updated_is_first {
            self.last_updated_is_first = false;
            return Ok(self.first_decoder.update_bits(bitstream)?);
        }
        else {
            self.last_updated_is_first = true;
            return Ok(self.second_decoder.update_bits(bitstream)?);
        }
        
    }

    fn reset(&mut self) {
        self.first_decoder.reset();
        self.second_decoder.reset();
        self.last_updated_is_first = false;
    }
}