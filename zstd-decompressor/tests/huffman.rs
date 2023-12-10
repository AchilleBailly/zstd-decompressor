mod huffman_test {
    use zstd_decompressor::decoders::huffman::{self, HuffmanDecoder};

    #[test]
    fn example_tree() {
        let example = huffman::build_example_tree();
        println!("{:?}", example); //should be smething like : HuffmanDecoder { 00: 65, 01: 67, 1: 66 }
        let widths: Vec<_> = std::iter::repeat(0).take(65).chain([2, 1, 2]).collect();
        println!("{:?}", HuffmanDecoder::from_number_of_bits(widths));
        let weights: Vec<_> = std::iter::repeat(0).take(65).chain([1, 2]).collect();
        println!("{:?}", HuffmanDecoder::from_weights(weights));
    }

    #[test]
    fn insert_example_tree_ok() {
        let mut example = HuffmanDecoder::Absent;

        let _should_be = HuffmanDecoder::Tree {
            left: Box::new(HuffmanDecoder::Tree {
                left: Box::new(HuffmanDecoder::Symbol { payload: b'A' }),
                right: Box::new(HuffmanDecoder::Absent),
            }),
            right: Box::new(HuffmanDecoder::Absent),
        };

        assert!(example.insert(b'A', 2));
        assert!(matches!(&example, _should_be));

        let _should_be = HuffmanDecoder::Tree {
            left: Box::new(HuffmanDecoder::Tree {
                left: Box::new(HuffmanDecoder::Symbol { payload: b'A' }),
                right: Box::new(HuffmanDecoder::Symbol { payload: b'C' }),
            }),
            right: Box::new(HuffmanDecoder::Absent),
        };
        assert!(example.insert(b'C', 2));
        assert!(matches!(&example, _should_be));

        let _should_be = HuffmanDecoder::Tree {
            left: Box::new(HuffmanDecoder::Tree {
                left: Box::new(HuffmanDecoder::Symbol { payload: b'A' }),
                right: Box::new(HuffmanDecoder::Symbol { payload: b'C' }),
            }),
            right: Box::new(HuffmanDecoder::Symbol { payload: b'C' }),
        };
        assert!(example.insert(b'B', 2));
        assert!(matches!(&example, _should_be));
    }
}
