use std::fmt::{self, Formatter};

use crate::{
    decoders::{alternating::AlternatingDecoder, fse::FseTable, BitDecoder},
    parsing::{BackwardBitParser, ForwardBitParser, ForwardByteParser},
    utils::discrete_log2,
};

use super::Result;

#[derive(PartialEq)]
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
            Some((HuffmanDecoder::Absent, _)) => self.next(),
            None => None,
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
                _ => unreachable!(),
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
        let tree = Self::from_weights(weights)?;

        Ok(tree)
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

        let mut bitstream = BackwardBitParser::new(&data[parser.bytes_read()..]).unwrap();

        let mut weights = Vec::new();
        let mut decoder = AlternatingDecoder::new(fse_table);
        decoder.initialize(&mut bitstream)?;

        while decoder.expected_bits() <= bitstream.len() {
            weights.push(decoder.symbol() as u8);
            decoder.update_bits(&mut bitstream)?;
        }
        weights.push(decoder.symbol() as u8);
        weights.push(decoder.symbol() as u8);
        // TODO: Verify that we have all weights : https://datatracker.ietf.org/doc/html/rfc8878#section-4.2.1.2

        Ok(weights)
    }

    pub fn insert(&mut self, symbol: u8, width: u8) -> bool {
        if width == 0 {
            match self {
                HuffmanDecoder::Absent => {
                    *self = HuffmanDecoder::Symbol { payload: symbol };
                    true
                }
                _ => false,
            }
        } else {
            match self {
                HuffmanDecoder::Tree {
                    left: gauche,
                    right: droite,
                } => gauche.insert(symbol, width - 1) || droite.insert(symbol, width - 1),
                HuffmanDecoder::Absent => {
                    *self = HuffmanDecoder::Tree {
                        left: Box::new(HuffmanDecoder::Absent),
                        right: Box::new(HuffmanDecoder::Absent),
                    };
                    self.insert(symbol, width)
                }
                HuffmanDecoder::Symbol { payload: _ } => {
                    panic!("Trying to inster a symbol into another")
                }
            }
        }
    }

    pub fn from_number_of_bits(numb_bytes: Vec<u8>) -> HuffmanDecoder {
        let mut symb: Vec<(u8, u8)> = vec![];
        for i in 0..numb_bytes.len() {
            if numb_bytes[i] != 0 {
                symb.push((i as u8, numb_bytes[i]))
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
        let mut sum: u32 = 0; //Pour se souvenir de la somme des poids connus
        for i in 0..weights.len() {
            if weights[i] != 0 {
                sum += 1 << (weights[i] - 1); //On calcule la somme
            }
        }
        let mut puissance: u8 = discrete_log2(sum); //On calcule la puissance
                                                    //ilog2 arrondit par valeur inférieure
        if 1 << puissance < sum {
            puissance += 1;
        }

        let manquant: u8 = ((1u32 << puissance) - sum) as u8;

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

    pub fn decode(&self, parser: &mut BackwardBitParser) -> Result<u8> {
        match self {
            HuffmanDecoder::Symbol { payload } => Ok(*payload),
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
