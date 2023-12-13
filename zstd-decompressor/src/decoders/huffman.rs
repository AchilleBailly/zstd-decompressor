use num_traits::pow;
use std::fmt::{self, Error, Formatter};

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
    type Item = Vec<(HuffmanDecoder, String)>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.noeuds.pop() {
            Some((HuffmanDecoder::Symbol { payload: symb }, prefixe)) => {
                Some(vec![(HuffmanDecoder::Symbol { payload: *symb }, prefixe)])
                // Tu recrées un élément, c'est pas super opti, tu peux direct renvoyer l'élément
            }
            Some((
                HuffmanDecoder::Tree {
                    left: gauche,
                    right: droite,
                },
                mut prefixe,
            )) => {
                self.noeuds.push((droite, {
                    prefixe.push('1');
                    prefixe.clone()
                }));
                self.noeuds.push((gauche, {
                    prefixe.push('0');
                    prefixe
                }));
                self.next()
            }
            _ => self.next(),
        }
    }
}

impl fmt::Debug for HuffmanDecoder {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let mut t = f.debug_struct("HuffmanDecoder");
        for noeud in vec![(self, " ")] {
            match noeud {
                (HuffmanDecoder::Absent, _) => (),
                (HuffmanDecoder::Symbol { payload: symb }, prefixe) => {
                    t.field(prefixe, symb);
                    ()
                }
                (
                    HuffmanDecoder::Tree {
                        left: gauche,
                        right: droite,
                    },
                    prefixe,
                ) => (),
            }
        }

        t.finish()
    }
}

impl HuffmanDecoder {
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
                HuffmanDecoder::Symbol { payload: _ } => return false ,
            }
        }
        return true;
    }

    pub fn from_number_of_bits(numb_bytes: Vec<u8>) -> HuffmanDecoder {
        let mut symb: Vec<(i32, u8)> = vec![];
        for i in 1..numb_bytes.len() {
            if numb_bytes[i] != 0 {
                symb.push((i.try_into().unwrap(), numb_bytes[i].try_into().unwrap()))
            }
        }
        symb.sort();
        symb.reverse();
        let mut res = HuffmanDecoder::Absent;
        for i in 1..symb.len() {
            res.insert(symb[i].0.try_into().unwrap(), symb[i].1);
        }
        return res;
    }

    pub fn from_weights(weights: Vec<u8>) -> HuffmanDecoder {
        let mut sum: i32 = 0;
        for i in 1..weights.len() {
            if weights[i] != 0 {
                sum += pow(2, (weights[i] - 1).into());
            }
        }
        let mut puissance: u8 = sum.ilog2().try_into().unwrap();
        //ilog2 arrondit par valeur inférieure
        puissance += 1;
        let manquant: u8 = ((pow(2, puissance.try_into().unwrap()) - sum).ilog2() + 1)
            .try_into()
            .unwrap();

        let mut prefixewidths: Vec<u8> = vec![];
        for i in 1..weights.len() {
            prefixewidths.push(puissance + 1 - weights[i]);
        }
        prefixewidths.push(puissance + 1 - manquant);

        return Self::from_number_of_bits(prefixewidths);
    }
}

pub fn build_example_tree() -> HuffmanDecoder {
    let mut tree = HuffmanDecoder::Absent;
    assert!(tree.insert(b'A', 2));
    assert!(tree.insert(b'C', 2));
    assert!(tree.insert(b'B', 1));
    tree
}
