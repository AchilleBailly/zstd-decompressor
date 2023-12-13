use crate::parsing::{self, ForwardBitParser};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error{"Parsing error: {0}"}]
    ParsingError(#[from] parsing::Error),
    #[error{"FSE table error: accuracy log {0} is too large."}]
    LargeAccuracyLog(u8),
    #[error{"FSE table is corrupted."}]
    CorruptedTable(),
}

type Result<T> = eyre::Result<T, Error>;

const MAX_AL: u8 = 9;

pub fn parse_fse_table(input: &mut ForwardBitParser) -> Result<(u8, Vec<i16>)> {
    let al = input.take(4)? as u8 + 5;
    if al > MAX_AL {
        return Err(Error::LargeAccuracyLog(al));
    }

    let table_size = 2u64.pow(al as u32);
    let mut distribution: Vec<i16> = Vec::new();
    let mut sum = 0;

    while sum < table_size {
        let max = table_size - sum;

        let bits_to_read = f64::log2((max + 2) as f64).ceil() as usize;
        let small_boundary = u64::pow(2, bits_to_read as u32) - max - 1;

        let peeked = input.peek(bits_to_read)?;
        let peeked_low = input.peek(bits_to_read - 1)?;

        let reuse;
        let decoded = if peeked_low < small_boundary && bits_to_read >= 4 {
            reuse = true;
            input.take(bits_to_read - 1)?
        } else if bits_to_read >= 4 {
            reuse = false;
            input.take(bits_to_read)? - small_boundary
        } else {
            reuse = false;
            input.take(bits_to_read)?
        };

        let proba = decoded as i16 - 1;
        println!(
            "{: >2} {: >6b} {} {: >5} {: >2}",
            max, peeked, bits_to_read, reuse, proba
        );

        if proba == -1 {
            sum += 1;
            distribution.push(proba);
        } else if proba == 0 {
            distribution.push(0);
            loop {
                let zeros = input.take(2)?;
                for _ in 0..zeros {
                    distribution.push(0);
                }

                if zeros != 3 {
                    break;
                }
            }
        } else {
            sum += proba as u64;
            distribution.push(proba);
        }
    }

    // TODO: verify that this implementation is the right (ie the test is fucked)

    if sum > table_size {
        return Err(Error::CorruptedTable());
    }

    return Ok((al, distribution));
}
