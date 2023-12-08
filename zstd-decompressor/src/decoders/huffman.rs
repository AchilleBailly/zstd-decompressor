mod huffman {

    enum HuffmanDecoder {
        Absent, 
        Symbol {payload : u8},
        Tree {left : HuffmanDecoder, right : HuffmanDecoder}

    }

    struct HuffmanDecoderIterator<'a> {
        noeuds : Vec<(&'a HuffmanDecoder, String)>
    }

    impl Iterator for HuffmanDecoderIterator {
        type Item = Vec<(&'a HuffmanDecoder, String)>;

        fn next(&mut self ) -> Option<Self::Item> {
            match self.noeuds.pop() {
                (HuffmanDecoder::Symbol(symb), prefixe) => Some(symb, prefixe),
                (HuffmanDecoder::Tree {gauche, droite}, prefixe) => {
                    self.add(droite, prefixe.push(String::from("1")));
                    self.add(gauche, prefixe.push(String::from("0")));
                    self.next;
                },
                _ => self.next,
            }
        }
    }

    impl fmt::Debug for HuffmanDecoder {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>{
            f.debug_struct("HuffmanDecoder");
            match self {
                HuffmanDecoder::Absent => f.finish(),
                HuffmanDecoder::Symbol(symb) => f.field(&self.payload),
                HuffmanDecoder::Tree(gauche, droite) => f.field(&self.gauche).field(&self.droite),
            }
            f.finish()
        }
    }

    impl HuffmanDecoder {
        pub fn insert(&mut self, symbol: u8, width: u8) -> bool {
            if width == 0{
                match self {
                    HuffmanDecoder::Absent => self = HuffmanDecoder::Symbol{Payload = symbol},
                    _ => return(false),
                }
            }
            else {
                match self {
                    HuffmanDecoder::Tree{gauche, droite} => if gauche.insert(symbol, width -1) != true {droite.insert(symbol, width -1)},
                    HuffmanDecoder::Absent => {self = HuffmanDecoder::Tree{left = Absent, right = Absent};
                                                self.insert(symbol, width - 1)}
                    HuffmanDecoder::Symbol => panic!(),
                }
            }
            return (true);
        }

        pub fn from_number_of_bits( numb_bytes : Vec<i32>) -> HuffmanDecoder{
            let symb : vec<(i32, u8)>;
            for i in len(numb_bytes) {
                if numb_bytes[i] != 0 {
                    symb.add((i, numb_bytes[i]))
                }
            }
            symb.1.sort();
            symb.reverse();
            let mut res = HuffmanDecoder::Absent;
            for i in len(symb) {
                res.insert(symb[i].0, symb[i].1);
            }
            return res;
        }

        pub fn from_weights (weights : Vec<u8>) -> HuffmanDecoder {
            let mut sum;
            for i in len(weights){
                if weights[i] != 0 {
                    sum += pow(2, weights[i] - 1);
                }
            }
            let puissance = sum.log2().ceil();
            let manquant = (pow(2,puissance) - sum).log2() + 1;
            
            let mut prefixewidths : Vec<u8>;
            for i in len(weights){
                prefixewidths.add(puissance + 1 - weights[i]);
            }
            prefixewidths.add(puissance + 1 - manquant);

            return from_number_of_bits(prefixewidths);

        }

        
    }

    fn build_example_tree() -> HuffmanDecoder {
        let mut tree = HuffmanDecoder::Absent;
        assert!(tree.insert(b'A', 2));
        assert!(tree.insert(b'C', 2));
        assert!(tree.insert(b'B', 1));
        tree
    }

}