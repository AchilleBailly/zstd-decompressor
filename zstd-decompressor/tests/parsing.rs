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
        todo!();
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
        todo!();
    }

    #[test]
    fn le_u32() {
        // Check that it returns the write value when enough bytes are present
        let mut parser = ForwardByteParser::new(&[0x00, 0x00, 0x00, 0x01, 0x10]);
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
