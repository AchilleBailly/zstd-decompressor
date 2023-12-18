use crate::{
    parsing::{BackwardBitParser, ForwardBitParser},
    utils::{discrete_log2, min_bits_required},
};
use std::{
    fmt::Debug,
    fmt::Display,
    ops::{Index, IndexMut},
};

use super::{BitDecoder, Error, Result};

const MAX_AL: u8 = 9;
const MAX_SYMBOL: usize = 256;

pub fn parse_fse_table(input: &mut ForwardBitParser) -> Result<(u8, Vec<i16>)> {
    let al = input.take(4)? as u8 + 5;
    if al > MAX_AL {
        return Err(Error::LargeAccuracyLog(al));
    }

    let mut distribution: Vec<i16> = Vec::new();
    let mut remaining: i32 = 1 << al;
    let mut n_sym = 0;

    while remaining > 0 && n_sym < MAX_SYMBOL {
        let bits_to_read = (discrete_log2(remaining + 1) + 1) as usize;

        let peeked = input.peek(bits_to_read)? as u16;

        let lower_mask = (1u16 << (bits_to_read - 1)) - 1;
        let threshold = (1u16 << bits_to_read) - 1 - (remaining as u16 + 1);

        let _reuse;
        let decoded = if (peeked & lower_mask) < threshold {
            _reuse = true;
            input.take(bits_to_read - 1)? as i16
        } else if peeked > lower_mask {
            _reuse = false;
            input.take(bits_to_read)? as i16 - threshold as i16
        } else {
            _reuse = false;
            input.take(bits_to_read)? as i16
        };

        let proba = decoded - 1;
        // println!(
        //     "{: >3} {: >9} {: >3} {: >5} {: >2}",
        //     remaining,
        //     format!("{:0>1$b}", peeked, bits_to_read),
        //     peeked,
        //     reuse,
        //     proba
        // );

        remaining -= proba.abs() as i32;
        distribution.push(proba);
        n_sym += 1;

        if proba == 0 {
            loop {
                let zeros = input.take(2)?;
                for _ in 0..zeros {
                    n_sym += 1;
                    distribution.push(0);
                }

                if zeros != 3 {
                    break;
                }
            }
        }
    }

    // TODO: verify that this implementation is the right (or if the test is fucked)
    // TODO: free memory once read, including partially read last byte

    if remaining != 0 || n_sym >= MAX_SYMBOL {
        return Err(Error::CorruptedTable);
    }

    return Ok((al, distribution));
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub output: u16,
    pub baseline: u16,
    pub bits_to_read: u8,
}

#[derive(Clone, Debug)]
struct TmpState {
    output: u16,
    baseline: Option<u16>,
    bits_to_read: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FseTable {
    pub table: Vec<State>,
    pub al: u8,
}

impl Display for FseTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "State Symbol   BL NB")?;
        self.table.iter().enumerate().for_each(|(pos, s)| {
            writeln!(
                f,
                "{: >5} {: >6} {: >4} {: >2}",
                format!("{:#04x}", pos),
                s.output,
                format!("{:#04x}", s.baseline),
                s.bits_to_read
            )
            .unwrap();
        });
        writeln!(f, "End of FSE table")
    }
}

impl FseTable {
    pub fn from_distribution(accuracy_log: u8, distribution: &[i16]) -> Result<Self> {
        if accuracy_log > MAX_AL {
            return Err(Error::LargeAccuracyLog(accuracy_log));
        }

        let num_symbol = distribution.len();
        let table_size = 1 << accuracy_log;
        let mut tmp_table: Vec<Option<TmpState>> = vec![None; table_size];

        // fill last elements with less than one probability symbols
        distribution
            .iter()
            .enumerate()
            .filter(|(_, &v)| v == -1) // Only retrieve less than one proba symbols
            .rev() // Reverse to have them in the same order as the end of tmp_table
            .zip(tmp_table.iter_mut().rev())
            .for_each(|(s, v)| {
                *v = Some(TmpState {
                    output: s.0 as u16,
                    baseline: Some(0),
                    bits_to_read: Some(accuracy_log),
                })
            });

        // Fill remaining spots with the non zero probability states
        let mut position = 0;
        distribution
            .iter()
            .enumerate()
            .filter(|(_, &v)| v > 0)
            .for_each(|(symbol, &proba)| {
                for i in 0..proba {
                    // each symbol is represented proba times in the table
                    tmp_table[position] = Some(TmpState {
                        output: symbol as u16,
                        baseline: None,
                        bits_to_read: None,
                    });

                    if symbol == distribution.len() - 1 && i == proba - 1 {
                        break;
                    }

                    while !matches!(tmp_table[position], None) {
                        position =
                            (position + (table_size >> 1) + (table_size >> 3) + 3) % table_size;
                    }
                }
            });

        // remove options
        let mut tmp_table = tmp_table
            .into_iter()
            .map(|v| match v {
                None => return Err(Error::CorruptedTable),
                Some(s) => Ok(s),
            })
            .collect::<Result<Vec<TmpState>>>()?;

        // Compute baseline and bits_to_read for non-zero proba symbols
        for symbol in 0..num_symbol as u16 {
            let mut grouped: Vec<_> = tmp_table
                .iter_mut()
                .filter(|s| s.output == symbol)
                .collect();
            let num_states = grouped.len();
            let parts = 1 << (f32::log2(num_states as f32).ceil() as u32);
            let base_width = table_size / parts;
            let base_nb = discrete_log2(base_width) as u8;
            let mut baseline = 0;

            for i in parts - num_states..parts {
                let new_i = i % num_states;
                let (add, mult) = if new_i != i { (1, 2) } else { (0, 1) };

                grouped[new_i].bits_to_read = Some((base_nb + add) as u8);
                grouped[new_i].baseline = Some(baseline);

                baseline += base_width as u16 * mult;
            }
        }

        Ok(FseTable {
            table: tmp_table
                .iter()
                .map(|tmp_s| State {
                    output: tmp_s.output,
                    bits_to_read: tmp_s.bits_to_read.unwrap(),
                    baseline: tmp_s.baseline.unwrap(),
                })
                .collect(),
            al: accuracy_log,
        })
    }

    pub fn parse(input: &mut ForwardBitParser) -> Result<Self> {
        let (al, distribution) = parse_fse_table(input)?;

        FseTable::from_distribution(al, &distribution)
    }

    pub fn al(&self) -> u8 {
        self.al
    }
}

impl Index<usize> for FseTable {
    type Output = State;

    fn index(&self, index: usize) -> &Self::Output {
        &self.table[index]
    }
}

impl IndexMut<usize> for FseTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.table[index]
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct FseDecoder {
    table: FseTable,
    cur_state: usize,
    next_symbol_s: u8,
    next_symbol: Option<u16>,
}

impl FseDecoder {
    pub fn new(input: &mut ForwardBitParser) -> Result<Self> {
        let table = FseTable::parse(input)?;
        let cur_state = 0;
        let next_symbol = None;
        let next_symbol_s = 0;

        Ok(FseDecoder {
            table,
            cur_state,
            next_symbol_s,
            next_symbol,
        })
    }

    pub fn new_from_table(table: FseTable) -> Self {
        let cur_state = 0;
        let next_symbol = None;
        let next_symbol_s = 0;

        FseDecoder {
            table,
            cur_state,
            next_symbol_s,
            next_symbol,
        }
    }

    // pub fn from(table: FseTable) -> Self {
    //     let cur_state = 0;
    //     let next_symbol = None;
    //     let next_symbol_s = 0;

    //     FseDecoder {
    //         table,
    //         cur_state,
    //         next_symbol_s,
    //         next_symbol,
    //     }
    // }
}

impl<'a> BitDecoder<u16> for FseDecoder {
    fn initialize(&mut self, bitstream: &mut BackwardBitParser) -> Result<()> {
        let state = bitstream.take(self.table.al() as usize)? as usize;

        self.next_symbol = Some(self.table[state].output);
        self.next_symbol_s = min_bits_required(self.next_symbol.unwrap());
        self.cur_state = state;

        Ok(())
    }

    fn expected_bits(&self) -> usize {
        self.table[self.cur_state].bits_to_read as usize
    }

    fn symbol(&mut self) -> u16 {
        if matches!(self.next_symbol, None) {
            panic!("Attempting to retrieve non set symbol, you may need to initialize or update first.");
        }

        let res = self.next_symbol.unwrap();
        self.next_symbol = None;

        res
    }

    fn update_bits(&mut self, bitstream: &mut BackwardBitParser) -> Result<bool> {
        if matches!(self.next_symbol, Some(..)) {
            panic!("Attempting to update without reading the symbol first.");
        }

        let new_state = bitstream.take(self.expected_bits())? as usize
            + self.table[self.cur_state].baseline as usize;

        self.next_symbol = Some(self.table[new_state].output);
        self.next_symbol_s = min_bits_required(self.next_symbol.unwrap());
        self.cur_state = new_state;

        Ok(false)
    }

    fn reset(&mut self) {
        self.next_symbol = None;
    }
}
