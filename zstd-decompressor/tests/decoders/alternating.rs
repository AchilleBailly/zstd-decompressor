#[cfg(test)]

mod alternating_tests {

    use zstd_decompressor::{
        decoders::alternating,
        decoders::{
            fse::{FseDecoder, FseTable, State},
            BitDecoder,
        },
        parsing::BackwardBitParser,
    };

    fn get_table() -> FseTable {
        FseTable {
            table: vec![
                State {
                    output: 0,
                    baseline: 1,
                    bits_to_read: 0,
                },
                State {
                    output: 3,
                    baseline: 2,
                    bits_to_read: 1,
                },
                State {
                    output: 1,
                    baseline: 0,
                    bits_to_read: 1,
                },
                State {
                    output: 0,
                    baseline: 2,
                    bits_to_read: 1,
                },
            ],
            al: 2,
        }
    }

    #[test]
    fn new_test_ok() {
        let table = get_table();
        let table_bis = table.clone();

        let alternating = alternating::AlternatingDecoder::new(table);

        let decodeur = FseDecoder::new_from_table(table_bis);
        let first = alternating.first_decoder;
        let second = alternating.second_decoder;
        assert_eq!(second, decodeur);
        assert_eq!(first, decodeur);
        assert!(!alternating.last_updated_is_first);
    }

    #[test]
    fn alternating_initialize_test() {
        let table = get_table();

        let mut alternating = alternating::AlternatingDecoder::new(table);
        // let data: &[u8; 3] = &[0b00111111, 0b11000000, 0b1100];
        let data: &[u8; 3] = &[0b1001_1_000, 0b0000_0001, 0b1111_1110];
        let mut parser = BackwardBitParser::new(data).unwrap();
        alternating.initialize(&mut parser).unwrap();
        let good_values = &[
            0, 0, 0, 0, 1, 1, 0, 0, 3, 3, 1, 1, 0, 0, 3, 3, 0, 0, 1, 1, 3, 3,
        ];

        for &v in good_values {
            assert_eq!(v, alternating.symbol());
            println!("{:?}", alternating.expected_bits());
            alternating.update_bits(&mut parser).unwrap();
        }
        println!("{}", parser.len());
    }

    #[test]
    fn ending_get_both_symbols_ok() {
        let table = get_table();

        let mut alternating = alternating::AlternatingDecoder::new(table);
        // let data: &[u8; 3] = &[0b0011_1111, 0b1100_0000, 0b1100];
        let data: &[u8; 3] = &[0b1001_1000, 0b0000_0001, 0b1111_1110];
        let mut parser = BackwardBitParser::new(data).unwrap();
        alternating.initialize(&mut parser).unwrap();

        while alternating.expected_bits() <= parser.len() {
            alternating.symbol();
            alternating.update_bits(&mut parser).unwrap();
        }

        // If we have exhausted all bits in the bitstream, we can still read the symbols that are stored in the decoder
        alternating.symbol();
        alternating.symbol();
    }
}
