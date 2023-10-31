#[cfg(test)]
mod frame_test {
    use zstd_decompressor::frame::{self, Frame};
    use zstd_decompressor::parsing::{self, ForwardByteParser};

    #[test]
    fn parse_skippable_frame() {
        let mut parser = ForwardByteParser::new(&[
            // Skippable frame with magic 0x184d2a53, length 3, content 0x10 0x20 0x30
            // and an extra byte at the end.
            0x53, 0x2a, 0x4d, 0x18, 0x03, 0x00, 0x00, 0x00, 0x10, 0x20, 0x30,
            0x40,
            //^--- magic (LE) ----^ ^------ 3 (LE) -------^ ^--- content ---^ ^-- extra
        ]);
        let Frame::SkippableFrame(skippable) = Frame::parse(&mut parser).unwrap() else {
            panic!("unexpected frame type")
        };
        assert_eq!(0x184d2a53, skippable.magic);
        assert_eq!(&[0x10, 0x20, 0x30], skippable.data);
        assert_eq!(1, parser.len());
    }

    #[test]
    fn error_on_unknown_frame() {
        let mut parser = ForwardByteParser::new(&[0x10, 0x20, 0x30, 0x40]);
        assert!(matches!(
            Frame::parse(&mut parser),
            Err(frame::Error::UnrecognizedMagic(0x40302010))
        ));
    }

    #[test]
    fn error_on_truncated_data_frame() {
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
}
