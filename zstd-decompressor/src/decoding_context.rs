use twox_hash::XxHash64;

use crate::{
    decoders::huffman::HuffmanDecoder, frame::MAX_WIN_SIZE, sequences::SymbolCompressionMode,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Window size is too big: max is {max} but got {got}"}]
    WindowSizeTooBig { max: u64, got: u64 },
    #[error{"Bad Offset value (0)"}]
    NullOffsetError,
}

pub struct DecodingContext {
    pub huffman_decoder: Option<HuffmanDecoder>,
    pub decoded: Vec<u8>,
    pub offsets: [usize; 3],
    pub window_size: u64,
    pub ll_repeat_decoder: Option<SymbolCompressionMode>,
    pub cmov_repeat_decoder: Option<SymbolCompressionMode>,
    pub ml_repeat_decoder: Option<SymbolCompressionMode>,
    pub checksum: Option<XxHash64>,
}

impl DecodingContext {
    pub fn new(window_size: u64) -> Result<Self, Error> {
        if window_size > MAX_WIN_SIZE {
            return Err(Error::WindowSizeTooBig {
                max: MAX_WIN_SIZE,
                got: window_size,
            });
        }

        Ok(DecodingContext {
            huffman_decoder: None,
            decoded: Vec::new(),
            offsets: [1, 4, 8],
            window_size,
            ll_repeat_decoder: None,
            cmov_repeat_decoder: None,
            ml_repeat_decoder: None,
            checksum: None,
        })
    }

    /// Decode an offset and properly maintain the three repeat offsets
    pub fn decode_offset(&mut self, offset: usize, literals_length: usize) -> Result<usize, Error> {
        match (offset, literals_length) {
            (0, _) => return Err(Error::NullOffsetError),
            (3, 0) => {
                self.offsets[2] = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] -= 1;
            }
            (3, _) => {
                let temp = self.offsets[2];
                self.offsets[2] = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = temp;
            }
            (2, 0) => {
                let temp = self.offsets[2];
                self.offsets[2] = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = temp;
            }
            (2, _) => {
                self.offsets.swap(0,1);
            }
            (1, 0) => {
                self.offsets.swap(0,1);
            }
            (1, _) => (),
            (_, _) => {
                self.offsets[2] = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = offset - 3;
            }
        }
        Ok(self.offsets[0])
    }

    /// Execute the sequences while updating the offsets
    pub fn execute_sequences(
        &mut self,
        sequences: Vec<(usize, usize, usize)>,
        literals: &[u8],
    ) -> Result<(), Error> {
        let mut literals_pos = 0;
        for (literals_copy, decoded_offset, n_offset_copy) in sequences.into_iter() {
            if decoded_offset == 0 {
                return Err(Error::NullOffsetError);
            }

            for _ in 0..literals_copy {
                self.decoded.push(literals[literals_pos]);
                literals_pos += 1;
            }

            let decoded_offset =
                self.decode_offset(decoded_offset, literals.len() - literals_pos)?;

            for _ in 0..n_offset_copy {
                self.decoded
                    .push(self.decoded[self.decoded.len() - decoded_offset]);
            }
        }

        for literal in literals.iter().skip(literals_pos) {
            self.decoded.push(*literal);
        }

        Ok(())
    }
}
