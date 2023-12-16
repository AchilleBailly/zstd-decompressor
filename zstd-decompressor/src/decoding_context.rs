use crate::{
    decoders::huffman::HuffmanDecoder,
    frame::{self, MAX_WIN_SIZE},
};

type Result<T> = eyre::Result<T, frame::Error>;

pub struct DecodingContext {
    pub huffman_decoder: Option<HuffmanDecoder>,
    pub decoded: Vec<u8>,
    pub window_size: u64,
}

impl DecodingContext {
    pub fn new(window_size: u64) -> Result<Self> {
        if window_size > MAX_WIN_SIZE {
            return Err(frame::Error::WindowSizeTooBig {
                max: MAX_WIN_SIZE,
                got: window_size,
            });
        }

        Ok(DecodingContext {
            huffman_decoder: None,
            decoded: Vec::new(),
            window_size: window_size,
        })
    }
}
