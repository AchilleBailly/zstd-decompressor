#[cfg(test)]
mod fse_decoder_tests {

    use zstd_decompressor::{decoders::fse, parsing::BitParser, parsing::ForwardBitParser};

    #[test]
    fn parse_fse_table_test_ok() {
        let data = &[0x30, 0x6f, 0x9b, 0x03];

        let mut parser = ForwardBitParser::new(data).unwrap();

        let (accuracy_log, table) = fse::parse_fse_table(&mut parser).unwrap();
        assert_eq!(5, accuracy_log);
        assert_eq!(&[18, 6, 2, 2, 2, 1, 1][..], &table);
        assert_eq!(parser.len(), 6);
    }

    #[test]
    fn fse_table_from_distribution_ok() {
        let table = fse::FseTable::from_distribution(5, &[18, 6, 2, 2, 2, 1, 1]).unwrap();

        assert!(matches!(
            table[0xc],
            fse::State {
                output: 1,
                baseline: 0x18,
                bits_to_read: 3
            }
        ));
    }

    #[test]
    fn fse_table_from_distribution2_ok() {
        let data: Vec<u8> = vec![
            0x21, 0x9d, 0x51, 0xcc, 0x18, 0x42, 0x44, 0x81, 0x8c, 0x94, 0xb4, 0x50, 0x1e,
        ];

        let mut parser = ForwardBitParser::new(data.as_slice()).unwrap();
        let table = fse::FseTable::parse(&mut parser).unwrap();

        assert!(matches!(
            table[0x3f],
            fse::State {
                output: 24,
                baseline: 0x10,
                bits_to_read: 4
            }
        ));

        assert!(matches!(
            table[0x2c],
            fse::State {
                output: 0,
                baseline: 0x34,
                bits_to_read: 2
            }
        ));
    }
}

#[cfg(test)]
mod fes_decoder_tests {
    use zstd_decompressor::{
        decoders::{
            fse::{FseDecoder, FseTable, State},
            BitDecoder,
        },
        parsing::{BitParser, ForwardBitParser},
    };

    #[test]
    fn run_full_decoder_ok() {
        let table = FseTable {
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
        };

        let data: &[u8; 2] = &[0b10000111, 0b10];
        let mut parser = ForwardBitParser::new(data).unwrap();

        let mut decoder = FseDecoder::from(table);
        decoder.initialize(&mut parser).unwrap();

        let good_values = &[0, 0, 1, 0, 3, 1, 0, 3, 0, 1, 3];

        for &v in good_values {
            assert_eq!(v, decoder.symbol());
            decoder.update_bits(&mut parser).unwrap();
        }
    }
}
