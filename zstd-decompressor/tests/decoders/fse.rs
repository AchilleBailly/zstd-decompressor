#[cfg(test)]
mod fse_decoder_tests {
    use zstd_decompressor::{decoders::fse::parse_fse_table, parsing::ForwardBitParser};

    #[test]
    fn parse_fse_table_test_ok() {
        let data = &[0x30, 0x6f, 0x9b, 0x03];
        let mut parser = ForwardBitParser::new(data);
        for i in 0..parser.len() {
            print!("{}", parser.take(1).unwrap());
            if (i + 1) % 4 == 0 {
                print!(" ");
            }
        }
        println!("");

        let mut parser = ForwardBitParser::new(data);

        let (accuracy_log, table) = parse_fse_table(&mut parser).unwrap();
        assert_eq!(5, accuracy_log);
        assert_eq!(&[18, 6, 2, 2, 2, 1, 1][..], &table);
        assert_eq!(parser.len(), 6);
    }
}
