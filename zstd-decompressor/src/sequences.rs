use crate::{
    decoders::{
        self,
        fse::{FseDecoder, FseTable},
        rle::RLEDecoder,
        sequence::SequenceDecoder,
        BitDecoder,
    },
    decoding_context::DecodingContext,
    parsing::{self, BackwardBitParser, ForwardBitParser, ForwardByteParser},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Parsing error: {0}"}]
    ParsingError(#[from] parsing::Error),
    #[error{"Corrupted data: reserved field set."}]
    ReservedSet,
    #[error{"Error in decoder: {0}"}]
    DecoderError(#[from] decoders::Error),
    #[error{"Corrupted data: repeat sequence mode with no previous decoder"}]
    NoPreviousDecoder,
}

type Result<T> = eyre::Result<T, Error>;

/// Default distribution for Literals Length, Offset and Match Length
/// Used to constrcut table for the corresponding the FSE Encoder
const LITERALS_LENGTH_DISTRI: [i16; 36] = [
    4, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 2, 1, 1, 1, 1, 1,
    -1, -1, -1, -1,
];
const OFFSET_DISTRI: [i16; 29] = [
    1, 1, 1, 1, 1, 1, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1,
];
const MATCH_LENGTH_DISTRI: [i16; 53] = [
    1, 4, 3, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1, -1, -1,
];

#[derive(Debug)]
pub struct Sequences<'a> {
    pub number_of_sequences: usize,
    pub literal_lengths_mode: SymbolCompressionMode,
    pub offsets_mode: SymbolCompressionMode,
    pub match_lengths_mode: SymbolCompressionMode,
    pub bitstream: &'a [u8],
}

impl<'a> Sequences<'a> {
    /// Parse the sequences data from the stream
    pub fn parse(input: &mut ForwardByteParser<'a>) -> Result<Self> {
        let num_seq = Self::parse_num_sequences(input)?;

        let mut seq = Sequences {
            number_of_sequences: num_seq,
            literal_lengths_mode: SymbolCompressionMode::RepeatMode,
            offsets_mode: SymbolCompressionMode::RepeatMode,
            match_lengths_mode: SymbolCompressionMode::RepeatMode,
            bitstream: &[],
        };

        if num_seq == 0 {
            return Ok(seq);
        }

        let mut modes = Self::parse_symbol_compression(input)?;

        seq.match_lengths_mode = modes.pop().unwrap();
        seq.offsets_mode = modes.pop().unwrap();
        seq.literal_lengths_mode = modes.pop().unwrap();
        seq.bitstream = input.slice(input.len())?;

        Ok(seq)
    }

    fn parse_num_sequences(input: &mut ForwardByteParser<'a>) -> Result<usize> {
        let byte0 = input.u8()?;

        Ok(match byte0 as usize {
            0 => 0,
            v if v < 128 => v,
            v if v < 255 => ((v - 128) << 8) + input.u8()? as usize,
            255 => input.u8()? as usize + ((input.u8()? as usize) << 8) + 0x7F,
            _ => unreachable!(),
        })
    }

    /// Parse symbol compression byte of sequences section
    /// Return Literal Length mode, Offsets mode and Match Lengths mode in that order
    fn parse_symbol_compression(
        input: &mut ForwardByteParser<'a>,
    ) -> Result<Vec<SymbolCompressionMode>> {
        let mut parser = ForwardBitParser::new(input.slice(1)?).unwrap();

        // reserved field, must be 0
        if parser.take(2).unwrap() != 0 {
            return Err(Error::ReservedSet);
        }

        let mut modes_tmp = [
            SymbolCompressionModeTmp::RepeatMode,
            SymbolCompressionModeTmp::RepeatMode,
            SymbolCompressionModeTmp::RepeatMode,
        ];
        for i in (0..3).rev() {
            modes_tmp[i] = match parser.take(2).unwrap() {
                0 => SymbolCompressionModeTmp::PredefinedMode,
                1 => SymbolCompressionModeTmp::RLEMode,
                2 => SymbolCompressionModeTmp::FseCompressedMode,
                3 => SymbolCompressionModeTmp::RepeatMode,
                _ => unreachable!(),
            }
        }

        let res = modes_tmp
            .iter()
            .map(|mode_tmp| {
                let mut new_data;
                match mode_tmp {
                    SymbolCompressionModeTmp::RepeatMode => Ok(SymbolCompressionMode::RepeatMode),
                    SymbolCompressionModeTmp::PredefinedMode => {
                        Ok(SymbolCompressionMode::PredefinedMode)
                    }
                    SymbolCompressionModeTmp::RLEMode => {
                        Ok(SymbolCompressionMode::RLEMode(input.u8()?))
                    }
                    SymbolCompressionModeTmp::FseCompressedMode => {
                        new_data = input.slice(input.len())?;
                        let mut parser = ForwardBitParser::new(new_data).unwrap();
                        let mode =
                            SymbolCompressionMode::FseCompressedMode(FseTable::parse(&mut parser)?);
                        new_data = &new_data[parser.bytes_read()..];
                        *input = ForwardByteParser::new(new_data);

                        Ok(mode)
                    }
                }
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(res)
    }

    /// Get the correct decoder given the context (which the stores the previous usable decoder)
    /// the SymbolCompressionMode and the code type
    fn get_decoder(
        code_type: CodeType,
        mode: SymbolCompressionMode,
        previous_decoder: &Option<SymbolCompressionMode>,
    ) -> Result<(Box<dyn BitDecoder<u16>>, SymbolCompressionMode)>
    where
        RLEDecoder: BitDecoder<u16>,
        FseDecoder: BitDecoder<u16>,
    {
        match mode {
            SymbolCompressionMode::RLEMode(b) => Ok((
                Box::new(RLEDecoder::new(b)),
                SymbolCompressionMode::RLEMode(b),
            )),
            SymbolCompressionMode::FseCompressedMode(table) => Ok((
                Box::new(FseDecoder::new_from_table(table.clone())),
                SymbolCompressionMode::FseCompressedMode(table),
            )),
            SymbolCompressionMode::RepeatMode => match previous_decoder {
                None => Err(Error::NoPreviousDecoder),
                Some(d) if matches!(d, &SymbolCompressionMode::RepeatMode) => {
                    Err(Error::NoPreviousDecoder) // We don't want infinit recursion
                }
                Some(d) => Self::get_decoder(code_type, d.clone(), &None),
            },
            SymbolCompressionMode::PredefinedMode => {
                let table = match code_type {
                    CodeType::LiteralsLength => {
                        FseTable::from_distribution(6, &LITERALS_LENGTH_DISTRI)?
                    }
                    CodeType::Offset => FseTable::from_distribution(5, &OFFSET_DISTRI)?,
                    CodeType::MatchLength => FseTable::from_distribution(6, &MATCH_LENGTH_DISTRI)?,
                };

                Ok((
                    Box::new(FseDecoder::new_from_table(table)),
                    SymbolCompressionMode::PredefinedMode,
                ))
            }
        }
    }

    /// Return vector of (literals length, offset value, match length) and update the
    /// decoding context with the tables if appropriate.
    pub fn decode(self, context: &mut DecodingContext) -> Result<Vec<(usize, usize, usize)>> {
        let (mut ll_decoder, new_ll_repeat) = Self::get_decoder(
            CodeType::LiteralsLength,
            self.literal_lengths_mode,
            &context.ll_repeat_decoder,
        )?;
        let (mut offset_decoder, new_cmov_repeat) = Self::get_decoder(
            CodeType::Offset,
            self.offsets_mode,
            &context.cmov_repeat_decoder,
        )?;
        let (mut match_decoder, new_ml_repeat) = Self::get_decoder(
            CodeType::MatchLength,
            self.match_lengths_mode,
            &context.ml_repeat_decoder,
        )?;

        let mut seq_decoder =
            SequenceDecoder::new(&mut *ll_decoder, &mut *offset_decoder, &mut *match_decoder);

        let mut parser = BackwardBitParser::new(self.bitstream)?;
        seq_decoder.initialize(&mut parser)?;

        let mut res = Vec::new();

        // TODO: verify that -1
        for _ in 0..self.number_of_sequences - 1 {
            seq_decoder.update_symbol_value(&mut parser)?;
            res.push(seq_decoder.symbol());
            seq_decoder.update_bits(&mut parser)?;
        }

        seq_decoder.update_symbol_value(&mut parser)?;
        res.push(seq_decoder.symbol());

        // Update the repeat decoder for each type
        context.cmov_repeat_decoder = Some(new_cmov_repeat);
        context.ll_repeat_decoder = Some(new_ll_repeat);
        context.ml_repeat_decoder = Some(new_ml_repeat);

        Ok(res)
    }
}

#[derive(Clone, Copy)]
pub enum SymbolCompressionModeTmp {
    PredefinedMode,
    RLEMode,
    FseCompressedMode,
    RepeatMode,
}

#[derive(Clone, Debug)]
pub enum CodeType {
    LiteralsLength,
    Offset,
    MatchLength,
}

#[derive(Clone, Debug)]
pub enum SymbolCompressionMode {
    PredefinedMode,
    RLEMode(u8),
    FseCompressedMode(FseTable),
    RepeatMode,
}
