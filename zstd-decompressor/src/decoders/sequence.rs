use crate::parsing::BackwardBitParser;

use super::{BitDecoder, Result};

pub struct SequenceDecoder<'d> {
    ll_code_decoder: &'d mut dyn BitDecoder<u16>,
    cmov_code_decoder: &'d mut dyn BitDecoder<u16>,
    ml_code_decoder: &'d mut dyn BitDecoder<u16>,
    ll_value: usize,
    offset_value: usize,
    match_value: usize,
}

impl<'d> SequenceDecoder<'d> {
    pub fn new(
        ll_code_decoder: &'d mut dyn BitDecoder<u16>,
        cmov_code_decoder: &'d mut dyn BitDecoder<u16>,
        ml_code_decoder: &'d mut dyn BitDecoder<u16>,
    ) -> Self {
        SequenceDecoder {
            ll_code_decoder,
            cmov_code_decoder,
            ml_code_decoder,
            ll_value: 0,
            offset_value: 0,
            match_value: 0,
        }
    }

    pub fn get_value(
        &self,
        code: u16,
        table: &[(u16, usize, usize)],
        bitstream: &mut BackwardBitParser,
    ) -> Result<usize> {
        let (_, baseline, nb_bits) = table.iter().filter(|v| v.0 == code).next().unwrap();

        Ok(bitstream.take(*nb_bits)? as usize + *baseline as usize)
    }
}

impl<'a> BitDecoder<(usize, usize, usize)> for SequenceDecoder<'a> {
    fn initialize(&mut self, bitstream: &mut BackwardBitParser) -> Result<()> {
        self.ll_code_decoder.initialize(bitstream)?;
        self.cmov_code_decoder.initialize(bitstream)?;
        self.ml_code_decoder.initialize(bitstream)?;

        Ok(())
    }

    fn expected_bits(&self) -> usize {
        self.ll_code_decoder.expected_bits()
            + self.ml_code_decoder.expected_bits()
            + self.cmov_code_decoder.expected_bits()
    }

    fn symbol(&mut self) -> (usize, usize, usize) {
        (self.ll_value, self.offset_value, self.match_value)
    }

    fn update_bits(&mut self, bitstream: &mut BackwardBitParser) -> Result<bool> {
        // Potentially reverse order of what is supposed to be !!!!!

        self.cmov_code_decoder.update_bits(bitstream)?;
        self.ml_code_decoder.update_bits(bitstream)?;
        self.ll_code_decoder.update_bits(bitstream)?;

        let offset_code = self.cmov_code_decoder.symbol();
        let ll_code = self.ll_code_decoder.symbol();
        let match_l_code = self.ml_code_decoder.symbol();

        self.ll_value = self.get_value(ll_code, &LL_CODE_TO_VALUE, bitstream)?;
        self.match_value = self.get_value(match_l_code, &ML_CODE_TO_VALUE, bitstream)?;
        self.offset_value =
            (1usize << offset_code) + bitstream.take(offset_code as usize)? as usize;
        if self.offset_value > 3 {
            self.offset_value -= 3;
        }

        todo!()
    }

    fn reset(&mut self) {
        todo!()
    }
}

const ML_CODE_TO_VALUE: [(u16, usize, usize); 53] = [
    (0, 3, 0),
    (1, 4, 0),
    (2, 5, 0),
    (3, 6, 0),
    (4, 7, 0),
    (5, 8, 0),
    (6, 9, 0),
    (7, 10, 0),
    (8, 11, 0),
    (9, 12, 0),
    (10, 13, 0),
    (11, 14, 0),
    (12, 15, 0),
    (13, 16, 0),
    (14, 17, 0),
    (15, 18, 0),
    (16, 19, 0),
    (17, 20, 0),
    (18, 21, 0),
    (19, 22, 0),
    (20, 23, 0),
    (21, 24, 0),
    (22, 25, 0),
    (23, 26, 0),
    (24, 27, 0),
    (25, 28, 0),
    (26, 29, 0),
    (27, 30, 0),
    (28, 31, 0),
    (29, 32, 0),
    (30, 33, 0),
    (31, 34, 0),
    (32, 35, 1),
    (33, 37, 1),
    (34, 39, 1),
    (35, 41, 1),
    (36, 43, 2),
    (37, 47, 2),
    (38, 51, 3),
    (39, 59, 3),
    (40, 67, 4),
    (41, 83, 4),
    (42, 99, 5),
    (43, 131, 7),
    (44, 259, 8),
    (45, 515, 9),
    (46, 1027, 10),
    (47, 2051, 11),
    (48, 4099, 12),
    (49, 8195, 13),
    (50, 16387, 14),
    (51, 32771, 15),
    (52, 65539, 16),
];

const LL_CODE_TO_VALUE: [(u16, usize, usize); 36] = [
    (0, 0, 0),
    (1, 1, 0),
    (2, 2, 0),
    (3, 3, 0),
    (4, 4, 0),
    (5, 5, 0),
    (6, 6, 0),
    (7, 7, 0),
    (8, 8, 0),
    (9, 9, 0),
    (10, 10, 0),
    (11, 11, 0),
    (12, 12, 0),
    (13, 13, 0),
    (14, 14, 0),
    (15, 15, 0),
    (16, 16, 1),
    (17, 18, 1),
    (18, 20, 1),
    (19, 22, 1),
    (20, 24, 2),
    (21, 28, 2),
    (22, 32, 3),
    (23, 40, 3),
    (24, 48, 4),
    (25, 64, 6),
    (26, 128, 7),
    (27, 256, 8),
    (28, 512, 9),
    (29, 1024, 10),
    (30, 2048, 11),
    (31, 4096, 12),
    (32, 8192, 13),
    (33, 16384, 14),
    (34, 32768, 15),
    (35, 65536, 16),
];
