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
    #[error{"Error while decoding sequence : impossible value decoded"}]
    ImpossibleValue,
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
            (3, _) | (2, 0) => {
                let temp = self.offsets[2];
                self.offsets[2] = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = temp;
            }
            (2, _) | (1, 0) => {
                self.offsets.swap(0, 1);
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
        mut literals: &[u8],
    ) -> Result<(), Error> {
        for (literal_length, decoded_offset, match_length) in sequences {
            let decoded_offset = self.decode_offset(decoded_offset, literal_length)?;

            if literal_length > literals.len()
                || decoded_offset > self.decoded.len() + literal_length
            {
                return Err(Error::ImpossibleValue);
            }

            self.decoded.extend_from_slice(&literals[..literal_length]);
            literals = &literals[literal_length..];

            for _ in 0..match_length {
                self.decoded
                    .push(self.decoded[self.decoded.len() - decoded_offset]);
            }
        }

        for literal in literals {
            self.decoded.push(*literal);
        }

        Ok(())
    }
}

#[test]
fn execute_sequences() {
    let mut context = DecodingContext::new(0x42).unwrap();
    context
        .execute_sequences(
            vec![(3, 5, 3), (2, 11, 1)],
            &[0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68],
        )
        .unwrap();
    assert_eq!(
        vec![0x61, 0x62, 0x63, 0x62, 0x63, 0x62, 0x64, 0x65, 0x61, 0x66, 0x67, 0x68],
        context.decoded
    );
}
