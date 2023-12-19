mod huffman_test {
    use zstd_decompressor::{
        decoders::huffman::{self, HuffmanDecoder},
        parsing::{BackwardBitParser, ForwardByteParser},
    };

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

        let should_be = HuffmanDecoder::Tree {
            left: Box::new(HuffmanDecoder::Tree {
                left: Box::new(HuffmanDecoder::Symbol { payload: b'A' }),
                right: Box::new(HuffmanDecoder::Absent),
            }),
            right: Box::new(HuffmanDecoder::Absent),
        };

        assert!(example.insert(b'A', 2));
        assert_eq!(example, should_be);

        let should_be = HuffmanDecoder::Tree {
            left: Box::new(HuffmanDecoder::Tree {
                left: Box::new(HuffmanDecoder::Symbol { payload: b'A' }),
                right: Box::new(HuffmanDecoder::Symbol { payload: b'C' }),
            }),
            right: Box::new(HuffmanDecoder::Absent),
        };
        assert!(example.insert(b'C', 2));
        assert_eq!(example, should_be);

        let should_be = HuffmanDecoder::Tree {
            left: Box::new(HuffmanDecoder::Tree {
                left: Box::new(HuffmanDecoder::Symbol { payload: b'A' }),
                right: Box::new(HuffmanDecoder::Symbol { payload: b'C' }),
            }),
            right: Box::new(HuffmanDecoder::Symbol { payload: b'B' }),
        };
        assert!(example.insert(b'B', 1));
        assert_eq!(example, should_be);
    }

    #[test]
    fn huffman_project_example() {
        // 0 repeated 65 times, 1, 2
        let weights: Vec<_> = std::iter::repeat(0).take(65).chain([1, 2]).collect();
        let decoder = HuffmanDecoder::from_weights(weights).unwrap();
        dbg!(&decoder);
        let mut parser = BackwardBitParser::new(&[0x97, 0x01]).unwrap();
        let mut result = String::new();
        while !parser.is_empty() {
            let decoded = decoder.decode(&mut parser).unwrap();
            result.push(decoded as char); // We know they are valid A, B, or C char
        }
        assert_eq!(result, "BABCBB");
    }

    #[test]
    fn parse_direct_stream_ok() {
        // 0 repeated 65 times, 1, 2 weights encoded with 2 weights per byte
        let mut weights = std::iter::repeat(0)
            .take(65)
            .chain([1, 2])
            .collect::<Vec<u8>>()
            .chunks(2)
            .map(|c| {
                let v1 = c[0] << 4;
                let v2 = if c.len() > 1 { c[1] & 0xf } else { 0 };

                v1 + v2
            })
            .collect::<Vec<_>>();
        weights.insert(0, 127 + 67); // Insert number of weights (must be >= 128) in the tree

        let mut parser = ForwardByteParser::new(&weights);
        let decoder = HuffmanDecoder::parse(&mut parser).unwrap();

        let mut parser = BackwardBitParser::new(&[0x97, 0x01]).unwrap();
        let mut result = String::new();
        while !parser.is_empty() {
            let decoded = decoder.decode(&mut parser).unwrap();
            result.push(decoded as char); // We know they are valid A, B, or C char
        }
        assert_eq!(result, "BABCBB");
    }
}
