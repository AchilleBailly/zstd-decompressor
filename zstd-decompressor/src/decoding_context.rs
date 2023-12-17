use crate::{
    decoders::huffman::HuffmanDecoder,
    frame::{self, MAX_WIN_SIZE, Error},
};





pub struct DecodingContext {
    pub huffman_decoder: Option<HuffmanDecoder>,
    pub decoded: Vec<u8>,
    pub offsets: Vec<usize>,
    pub window_size: u64,
}

impl DecodingContext {
    pub fn new(window_size: u64) -> Result<Self, Error> {
        if window_size > MAX_WIN_SIZE {
            return Err(frame::Error::WindowSizeTooBig {
                max: MAX_WIN_SIZE,
                got: window_size,
            });
        }

        Ok(DecodingContext {
            huffman_decoder: None,
            decoded: Vec::new(),
            offsets : vec![1, 4, 8],
            window_size: window_size,
        })
    }

    /// Decode an offset and properly maintain the three repeat offsets
    pub fn decode_offset(&mut self, offset: usize, literals_length: usize) -> Result<usize, Error>{
        match (offset, literals_length) {
            (0, _) => return Err(Error::NullOffsetError),
            (3, 0) => {self.offsets[2] = self.offsets[1];
                    self.offsets[1] = self.offsets[0];
                    self.offsets[0] = self.offsets[0] - 1;},
            (3, _) => {let temp = self.offsets[2];
                self.offsets[2] = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = temp;},
            (2, 0) => {let temp = self.offsets[2];
                    self.offsets[2] = self.offsets[1];
                    self.offsets[1] = self.offsets[0];
                    self.offsets[0] = temp;},
            (2, _) => {let temp = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = temp;},
            (1, 0) => {let temp = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = temp;},
            (1, _) => (),
            (_, _) => {self.offsets[2] = self.offsets[1];
                self.offsets[1] = self.offsets[0];
                self.offsets[0] = offset - 3;},
        }
        return Ok(self.offsets[1])
    }

    /// Execute the sequences while updating the offsets
    pub fn execute_sequences(&mut self, sequences: Vec<(usize, usize, usize)>, literals: &[u8],) -> Result<(), Error> {
        let mut res: String;
        let mut pos_literals = 0;
        for seq in sequences {
            res.push(literals[pos_literals..seq.0] as char);
            pos_literals += seq.0;
            for i in 0..seq.0 {
                res.push(res[res.len()-seq.0] as char)
            }
            self.decode_offset(seq.1, literals.len());
        }
        return Ok(res);
    }
}

