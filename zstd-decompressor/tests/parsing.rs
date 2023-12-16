#[cfg(test)]
mod forward_byte_parser_tests {
    use zstd_decompressor::parsing::{self, ForwardByteParser};

    #[test]
    fn u8() {
        // Check that bytes are delivered in order
        let mut parser = ForwardByteParser::new(&[0x12, 0x23, 0x34]);
        assert_eq!(0x12, parser.u8().unwrap());
        assert_eq!(0x23, parser.u8().unwrap());
        assert_eq!(0x34, parser.u8().unwrap());
        assert!(matches!(
            parser.u8(),
            Err(parsing::Error::NotEnoughBytes {
                requested: 1,
                available: 0,
            })
        ));
    }

    #[test]
    fn len() {
        let parser = ForwardByteParser::new(&[0x12, 0x23, 0x34]);
        assert!(parser.len() == 3);

        let parser = ForwardByteParser::new(&[]);
        assert!(parser.len() == 0);
    }

    #[test]
    fn is_empty() {
        let arr = [0x12, 0x23, 0x34];
        let parser = ForwardByteParser::new(&arr);
        assert_eq!(false, parser.is_empty());
        assert_eq!(0x12, arr[0]);

        let parser = ForwardByteParser::new(&[]);
        assert_eq!(true, parser.is_empty());
    }

    #[test]
    fn slice() {
        let mut parser = ForwardByteParser::new(&[0x12, 0x23, 0x34]);
        let s = parser.slice(2).unwrap();
        assert_eq!(s, &[0x12, 0x23]);

        assert!(matches!(
            parser.slice(4),
            Err(parsing::Error::NotEnoughBytes {
                requested: 4,
                available: 1,
            })
        ));
    }

    #[test]
    fn le_u32() {
        // Check that it returns the write value when enough bytes are present
        let mut parser = ForwardByteParser::new(&[0x01, 0x00, 0x00, 0x00, 0x10]);
        assert_eq!(1u32, parser.le_u32().unwrap());
        assert_eq!(0x10, parser.u8().unwrap());

        // Check that when not enough bytes are present, an error is returned
        let mut parser = ForwardByteParser::new(&[0x00, 0x00, 0x00]);
        assert!(matches!(
            parser.le_u32(),
            Err(parsing::Error::NotEnoughBytes {
                requested: 4,
                available: 3,
            })
        ));
        assert_eq!(3, parser.len());
    }
}

#[cfg(test)]
mod forward_bit_parser_tests {
    use zstd_decompressor::parsing::{self, BitParser};

    #[test]
    fn is_empty_ok() {
        let data: &[u8] = &[];

        let parser = parsing::ForwardBitParser::new(data);

        assert!(parser.is_empty());
    }

    #[test]
    fn is_empty_nok() {
        let data: &[u8] = &[1];

        let parser = parsing::ForwardBitParser::new(data);

        assert!(!parser.is_empty());
    }

    #[test]
    fn len_ok() {
        let data: &[u8] = &[];

        let parser = parsing::ForwardBitParser::new(data);

        assert!(parser.len() == 0);
    }

    #[test]
    fn len_ok2() {
        let data: &[u8] = &[1];

        let parser = parsing::ForwardBitParser::new(data);

        assert!(parser.len() == 8);
    }

    #[test]
    fn take_whole_byte_ok() {
        let data = &[75];

        let mut parser = parsing::ForwardBitParser::new(data);

        assert_eq!(75, parser.take(8).unwrap());
        assert!(parser.is_empty());
    }

    #[test]
    fn take_whole_byte_and_half_ok() {
        let data = &[75, 0b0000_1111];

        let mut parser = parsing::ForwardBitParser::new(data);

        assert_eq!((15 << 8) + 75, parser.take(12).unwrap());
        assert!(parser.len() == 4);
    }

    #[test]
    fn take_few_ok() {
        let data = &[0b0101_1010, 0b1100_0011];

        let mut parser = parsing::ForwardBitParser::new(data);

        assert_eq!(0b010, parser.take(3).unwrap());
        assert_eq!(0b011, parser.take(3).unwrap());
        assert_eq!(0b1101, parser.take(4).unwrap());
        assert_eq!(0b110000, parser.take(6).unwrap());
        assert!(parser.is_empty());
    }

    #[test]
    fn take_more_than_64_nok() {
        let data = &[1; 10];

        let mut parser = parsing::ForwardBitParser::new(data);

        assert!(matches!(
            parser.take(67),
            Err(parsing::Error::MaximumReadableBitsExceeded(67))
        ));
        assert_eq!(80, parser.len());
    }

    #[test]
    fn take_more_than_available_nok() {
        let data = &[1; 6];

        let mut parser = parsing::ForwardBitParser::new(data);

        assert!(matches!(
            parser.take(60),
            Err(parsing::Error::NotEnoughBits {
                requested: 60,
                available: 48
            })
        ));
        assert_eq!(48, parser.len());
    }
}

mod backward_bit_parser_tests {
    use zstd_decompressor::{decoders::huffman::HuffmanDecoder, parsing::BackwardBitParser};

#[test]
fn huffman_project_example() {
    // 0 repeated 65 times, 1, 2
    let weights: Vec<_> = std::iter::repeat(0).take(65).chain([1, 2]).collect();
    let decoder = HuffmanDecoder::from_weights(weights).unwrap();
    let mut parser = BackwardBitParser::new(&[0x97, 0x01]).unwrap();
    let mut result = String::new();
    while !parser.is_empty() {
        let decoded = decoder.decode(&mut parser).unwrap();
        result.push(decoded as char);  // We know they are valid A, B, or C char
    }
    assert_eq!(result, "BABCBB");
}

}
