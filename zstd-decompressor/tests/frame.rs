#[cfg(test)]
mod frame_test {
    use zstd_decompressor::frame::{self, Frame};
    use zstd_decompressor::parsing::{self, ForwardByteParser};

    fn get_valid_skippable_parser() -> ForwardByteParser<'static> {
        return ForwardByteParser::new(&[
            // Skippable frame with magic 0x184d2a53, length 3, content 0x10 0x20 0x30
            // and an extra byte at the end.
            0x53, 0x2a, 0x4d, 0x18, 0x03, 0x00, 0x00, 0x00, 0x10, 0x20, 0x30,
            0x40,
            //^--- magic (LE) ----^ ^------ 3 (LE) -------^ ^--- content ---^ ^-- extra
        ]);
    }

    #[test]
    fn parse_skippable_frame() {
        let mut parser = get_valid_skippable_parser();
        let Frame::SkippableFrame(skippable) = Frame::parse(&mut parser).unwrap() else {
            panic!("unexpected frame type")
        };
        assert_eq!(0x184d2a53, skippable.magic);
        assert_eq!(&[0x10, 0x20, 0x30], skippable.data);
        assert_eq!(1, parser.len());
    }

    #[test]
    fn parsing_error_on_unknown_frame() {
        let mut parser = ForwardByteParser::new(&[0x10, 0x20, 0x30, 0x40]);
        assert!(matches!(
            Frame::parse(&mut parser),
            Err(frame::Error::UnrecognizedMagic(0x40302010))
        ));
    }

    #[test]
    fn parsing_error_on_truncated_data_frame() {
        let mut parser = ForwardByteParser::new(&[
            // Skippable frame with magic 0x184d2a53, length 3, content 0x10 0x20 which
            // does not have required length of 3
            0x53, 0x2a, 0x4d, 0x18, 0x03, 0x00, 0x00, 0x00, 0x10,
            0x20,
            //^--- magic (LE) ----^ ^------ 3 (LE) -------^ ^--- content ---^ ^-- extra
        ]);
        let res = Frame::parse(&mut parser);

        assert!(matches!(
            res,
            Err(frame::Error::ParsingError(parsing::Error::NotEnoughBytes {
                requested: 3,
                available: 2
            }))
        ));
        assert_eq!(parser.len(), 2); // did not read the truncated data
    }

    #[test]
    fn parsing_error_on_truncated_length_frame() {
        let mut parser = ForwardByteParser::new(&[
            // Skippable frame with magic 0x184d2a53, length 3 but truncated u32, no content
            0x53, 0x2a, 0x4d, 0x18, 0x03, 0x00,
            0x00,
            //^--- magic (LE) ----^ ^------ 3 (LE) -------^
        ]);
        let res = Frame::parse(&mut parser);

        assert!(matches!(
            res,
            Err(frame::Error::ParsingError(parsing::Error::NotEnoughBytes {
                requested: 4,
                available: 3
            }))
        ));
        assert_eq!(parser.len(), 3);
    }

    #[test]
    fn parsing_error_on_truncated_magic_frame() {
        let mut parser = ForwardByteParser::new(&[
            // Skippable frame with magic truncated magic 0x184d2a53
            0x53, 0x2a, 0x4d,
            //^--- magic (LE) ----^
        ]);
        let res = Frame::parse(&mut parser);

        assert!(matches!(
            res,
            Err(frame::Error::ParsingError(parsing::Error::NotEnoughBytes {
                requested: 4,
                available: 3
            }))
        ));
        assert_eq!(parser.len(), 3);
    }

    #[test]
    fn decode_skippable_frame_test() {
        let mut parser = get_valid_skippable_parser();
        let frame = Frame::parse(&mut parser).unwrap();

        assert_eq!(frame.decode().unwrap(), vec![0x10, 0x20, 0x30]);
    }
}

#[cfg(test)]
pub mod frame_iterator_tests {
    use zstd_decompressor::parsing::ForwardByteParser;

    fn get_valid_skippable_parser() -> ForwardByteParser<'static> {
        return ForwardByteParser::new(&[
            0x53, 0x2a, 0x4d, 0x18, 0x03, 0x00, 0x00, 0x00, 0x10, 0x20, 0x30, 0x51, 0x2a, 0x4d,
            0x18, 0x04, 0x00, 0x00, 0x00, 0x10, 0x20, 0x30, 0x40,
        ]);
    }

    #[test]
    fn next_test() {
        let parser = get_valid_skippable_parser();
        let mut iter = parser.iter();
        assert_eq!(
            iter.next().unwrap().unwrap().decode().unwrap(),
            vec![0x10, 0x20, 0x30]
        );
        assert_eq!(
            iter.next().unwrap().unwrap().decode().unwrap(),
            vec![0x10, 0x20, 0x30, 0x40]
        );
    }
}
