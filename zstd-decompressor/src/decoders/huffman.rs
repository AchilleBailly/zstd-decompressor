use eyre;
use num_traits::pow;
use std::fmt::{self, Formatter};
use thiserror;

use crate::{
    decoders::{alternating::AlternatingDecoder, fse::FseTable, BitDecoder},
    parsing::{self, BackwardBitParser, BitParser, ForwardBitParser, ForwardByteParser},
};

use super::{alternating, fse};

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error{"Error during the Backward parsing"}]
    ParserError,
    #[error{"Bad input data"}]
    InputDataError,
    #[error{"Parsing error:"}]
    ParsingError(#[from] parsing::Error),
    #[error{"Error while decoding FSE table: {0}"}]
    FseParsingError(#[from] fse::Error),
    #[error{"Error while decoding: {0}"}]
    FseDecoderError(#[from] alternating::Error),
}

pub type Result<T> = eyre::Result<T, DecodeError>;

pub enum HuffmanDecoder {
    Absent,
    Symbol {
        payload: u8,
    },
    Tree {
        left: Box<HuffmanDecoder>,
        right: Box<HuffmanDecoder>,
    },
}

struct HuffmanDecoderIterator<'a> {
    noeuds: Vec<(&'a HuffmanDecoder, String)>,
}

impl Iterator for HuffmanDecoderIterator<'_> {
    type Item = (HuffmanDecoder, String);

    fn next(&mut self) -> Option<Self::Item> {
        match self.noeuds.pop() {
            Some((HuffmanDecoder::Symbol { payload: symb }, prefixe)) => {
                Some((HuffmanDecoder::Symbol { payload: *symb }, prefixe))
                // Tu recrées un élément, c'est pas super opti, tu peux direct renvoyer l'élément
            }
            Some((
                HuffmanDecoder::Tree {
                    left: gauche,
                    right: droite,
                },
                mut prefixe,
            )) => {
                let mut old_prefix = prefixe.clone();
                self.noeuds.push((droite, {
                    prefixe.push('1');
                    prefixe
                }));
                self.noeuds.push((gauche, {
                    old_prefix.push('0');
                    old_prefix
                }));
                self.next()
            }
            _ => None,
        }
    }
}

impl fmt::Debug for HuffmanDecoder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let tree_iter = HuffmanDecoderIterator {
            noeuds: vec![(self, String::from(" "))],
        };
        let mut t = f.debug_struct("HuffmanDecoder");
        for noeud in tree_iter {
            match noeud.0 {
                HuffmanDecoder::Symbol { payload } => {
                    t.field(&noeud.1, &payload);
                }
                _ => (),
            }
        }

        t.finish()
    }
}

impl HuffmanDecoder {
    pub fn parse(input: &mut ForwardByteParser) -> Result<Self> {
        let header = input.u8()?;
        let weights = if header < 128 {
            Self::parse_fse(input, header)?
        } else {
            Self::parse_direct(input, header as usize - 127)?
        };
        Self::from_weights(weights)
    }

    fn parse_direct(input: &mut ForwardByteParser, num_weights: usize) -> Result<Vec<u8>> {
        let data = input.slice(num_weights / 2 + num_weights % 2)?; // 2 weights per byte

        let mut res = vec![];
        let mut parser = ForwardBitParser::new(data).unwrap();
        while !parser.is_empty() {
            let tmp = parser.take(4).unwrap() as u8;
            res.push(parser.take(4).unwrap() as u8);
            res.push(tmp);
        }

        res.truncate(num_weights);

        Ok(res)
    }

    fn parse_fse(input: &mut ForwardByteParser, compressed_size: u8) -> Result<Vec<u8>> {
        let data = input.slice(compressed_size as usize)?;

        let mut parser = ForwardBitParser::new(data).unwrap();

        let fse_table = FseTable::parse(&mut parser)?;

        let mut bitstream = ForwardBitParser::new(&data[parser.bytes_read()..]).unwrap();

        let mut weights = Vec::new();
        let mut decoder = AlternatingDecoder::new(fse_table);

        while decoder.expected_bits() < bitstream.len() {
            weights.push(decoder.symbol() as u8);
            decoder.update_bits(&mut bitstream)?;
        }
        weights.push(decoder.symbol() as u8);
        weights.push(decoder.symbol() as u8);

        Ok(weights)
    }

    pub fn insert(&mut self, symbol: u8, width: u8) -> bool {
        if width == 0 {
            match self {
                HuffmanDecoder::Absent => *self = HuffmanDecoder::Symbol { payload: symbol },
                _ => return false,
            }
        } else {
            match self {
                HuffmanDecoder::Tree {
                    left: gauche,
                    right: droite,
                } => {
                    if gauche.insert(symbol, width - 1) != true {
                        droite.insert(symbol, width - 1);
                    }
                }
                HuffmanDecoder::Absent => {
                    *self = HuffmanDecoder::Tree {
                        left: Box::new(HuffmanDecoder::Absent),
                        right: Box::new(HuffmanDecoder::Absent),
                    };
                    self.insert(symbol, width);
                }
                HuffmanDecoder::Symbol { payload: _ } => return false,
            }
        }
        return true;
    }

    pub fn from_number_of_bits(numb_bytes: Vec<u8>) -> HuffmanDecoder {
        let mut symb: Vec<(u8, u8)> = vec![];
        for i in 1..numb_bytes.len() {
            if numb_bytes[i] != 0 {
                symb.push((i.try_into().unwrap(), numb_bytes[i].try_into().unwrap()))
            }
        }
        symb.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0).reverse()));
        symb.reverse();
        let mut res = HuffmanDecoder::Absent;
        for i in 0..symb.len() {
            res.insert(symb[i].0, symb[i].1);
        }
        return res;
    }

    pub fn from_weights(weights: Vec<u8>) -> Result<HuffmanDecoder> {
        let mut sum: i32 = 0; //Pour se souvenir de la somme des poids connus
        for i in 1..weights.len() {
            if weights[i] != 0 {
                sum += pow(2, (weights[i] - 1).into()); //On calcule la somme
            }
        }
        let mut puissance: u8 = sum.ilog2().try_into().unwrap(); //On calcule la puissance
                                                                 //ilog2 arrondit par valeur inférieure
        if pow(2, puissance.into()) < sum {
            puissance += 1;
        }

        let manquant: u8 = ((pow(2, puissance.try_into().unwrap()) - sum).ilog2() + 1) //On calcule la puissance de deux manquante dans les poids
            .try_into()
            .unwrap();

        let mut prefixewidths: Vec<u8> = vec![];
        for i in 0..weights.len() {
            if weights[i] != 0 {
                prefixewidths.push(puissance + 1 - weights[i]);
            } else {
                prefixewidths.push(0);
            }
        }
        prefixewidths.push(puissance + 1 - manquant);

        return Ok(Self::from_number_of_bits(prefixewidths));
    }

    pub fn decode(&self, parser: &mut BackwardBitParser) -> Result<char> {
        match self {
            HuffmanDecoder::Symbol { payload } => Ok(*payload as char),
            HuffmanDecoder::Tree { left, right } => {
                let bit = parser.take(1)?;
                match bit {
                    0 => return left.decode(parser),
                    1 => return right.decode(parser),
                    _ => unreachable!(),
                }
            }
            HuffmanDecoder::Absent => panic!(),
        }
    }
}

pub fn build_example_tree() -> HuffmanDecoder {
    let mut tree = HuffmanDecoder::Absent;
    assert!(tree.insert(b'A', 2));
    assert!(tree.insert(b'C', 2));
    assert!(tree.insert(b'B', 1));
    tree
}
