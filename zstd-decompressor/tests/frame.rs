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

    fn get_valid_z_standard_frame_parser() -> ForwardByteParser<'static> {
        ForwardByteParser::new(&[
            0x28,
            0xB5,
            0x2F,
            0xFD,
            0b01_1_0_0_1_00, // single segment, so FCS is 1 byte, w/ checksum
            0x04,            // frame content size <
            0x21,            // block header: raw block and last one
            0x0,
            0x0,  // <
            0x10, // block content
            0x20,
            0x30,
            0x40, //<
            0x01, // content checksum of value 1
            0x00,
            0x00,
            0x00, //<
            0x42, // additionnal byte
        ])
    }

    #[test]
    fn parse_skippable_frame_ok() {
        let mut parser = get_valid_skippable_parser();
        let Frame::SkippableFrame(skippable) = Frame::parse(&mut parser).unwrap() else {
            panic!("unexpected frame type")
        };
        assert_eq!(0x184d2a53, skippable.magic);
        assert_eq!(&[0x10, 0x20, 0x30], skippable.data);
        assert_eq!(1, parser.len());
    }

    #[test]
    fn parse_standard_frame_ok() {
        let mut parser = get_valid_z_standard_frame_parser();
        let Frame::ZStandardFrame(standard) = Frame::parse(&mut parser).unwrap() else {
            panic!("Unexpected frame type")
        };

        assert_eq!(Some(1), standard.checksum());
        assert_eq!(1, standard.blocks().len());
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
    fn decode_skippable_frame_test() {
        let mut parser = get_valid_skippable_parser();
        let frame = Frame::parse(&mut parser).unwrap();

        assert_eq!(frame.decode().unwrap(), vec![0x10, 0x20, 0x30]);
    }
}

#[cfg(test)]
pub mod skippable_frame_tests {
    use zstd_decompressor::{
        frame::{self, Frame},
        parsing::{self, ForwardByteParser},
    };

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
}

#[cfg(test)]
pub mod frame_header_tests {
    use zstd_decompressor::frame::{Error, FrameHeader};
    use zstd_decompressor::parsing::ForwardByteParser;

    #[test]
    fn simple_valid_data_ok() {
        let mut parser = ForwardByteParser::new(&[
            0b01_1_0_0_0_00,
            0xcc, // Only frame content size present in 1 byte, window descriptor null so s
                  // ingle segment flag is 1 and window size will equal to frame content size
        ]);

        let h = FrameHeader::parse(&mut parser).unwrap();

        assert_eq!(h.content_checksum_flag, false, "Content checksum is not OK");
        assert_eq!(h.window_size, 0xcc, "Window size is not OK");
        assert_eq!(h.content_size, Some(0xcc), "Content size is not OK");
        assert_eq!(h.dictionnary_id, None, "Dictionnary ID is not OK");
    }

    #[test]
    fn simple_valid_data_2_ok() {
        let mut parser = ForwardByteParser::new(&[
            0b01_0_0_0_0_00,
            0x00,
            0xcc,
            0xdd, // Only frame content size present in 1 byte, window descriptor byte is set so s
                  // ingle segment flag is 0 and window size will equal to frame content size
                  // because we have 2 bytes for content size, we have to add 256 to the read value
        ]);

        let h = FrameHeader::parse(&mut parser).unwrap();

        assert_eq!(h.content_checksum_flag, false, "Content checksum is not OK");
        assert_eq!(h.window_size, 1024, "Window size is not OK");
        assert_eq!(h.content_size, Some(0xddcc + 256), "Content size is not OK");
        assert_eq!(h.dictionnary_id, None, "Dictionnary ID is not OK");
    }

    #[test]
    fn reserved_bit_set_should_throw_error() {
        let mut parser = ForwardByteParser::new(&[
            0b01_0_0_1_0_00, // reserved is set so should throw error
        ]);

        let h = FrameHeader::parse(&mut parser);

        let _ret = Error::ReservedSet(String::from("FrameHeader"));
        assert!(matches!(h, Err(_ret)));
    }

    #[test]
    fn simple_valid_data_with_dict_id_ok() {
        let mut parser = ForwardByteParser::new(&[
            0b01_0_0_0_0_10,
            0x00, // window descriptor
            0xef, // dict Id
            0xab, // dict Id
            0xcc,
            0xdd, // Only frame content size present in 1 byte, window descriptor byte is set so s
                  // ingle segment flag is 0 and window size will equal to frame content size
                  // because we have 2 bytes for content size, we have to add 256 to the read value
        ]);

        let h = FrameHeader::parse(&mut parser).unwrap();

        assert_eq!(h.content_checksum_flag, false, "Content checksum is not OK");
        assert_eq!(h.window_size, 1024, "Window size is not OK");
        assert_eq!(h.content_size, Some(0xddcc + 256), "Content size is not OK");
        assert_eq!(h.dictionnary_id, Some(0xabef), "Dictionnary ID is not OK");
    }

    #[test]
    fn simple_valid_data_long_dict_and_fc_ok() {
        let mut parser = ForwardByteParser::new(&[
            0b11_0_0_0_0_11,
            0x00, // window descriptor
            0xef, // dict Id
            0xab, // dict Id
            0xef, // dict Id
            0xab, // dict Id
            0xcc,
            0xdd,
            0xcc,
            0xdd,
            0xcc,
            0xdd,
            0xcc,
            0xdd, // Only frame content size present in 1 byte, window descriptor byte is set so s
                  // ingle segment flag is 0 and window size will equal to frame content size
                  // because we have 2 bytes for content size, we have to add 256 to the read value
        ]);

        let h = FrameHeader::parse(&mut parser).unwrap();

        assert_eq!(h.content_checksum_flag, false, "Content checksum is not OK");
        assert_eq!(h.window_size, 1024, "Window size is not OK");
        assert_eq!(
            h.content_size,
            Some(0xddccddccddccddcc),
            "Content size is not OK"
        );
        assert_eq!(
            h.dictionnary_id,
            Some(0xabefabef),
            "Dictionnary ID is not OK"
        );
    }
}

#[cfg(test)]
pub mod z_standard_frame_tests {
    use zstd_decompressor::parsing::ForwardByteParser;
    use zstd_decompressor::{frame, parsing};

    // only need to test errors cases as this only builds upon other tested functions
    #[test]
    fn parse_with_checksum_ok() {
        let mut parser = ForwardByteParser::new(&[
            0b01_1_0_0_1_00, // single segment, so FCS is 1 byte, w/ checksum
            0x04,            // frame content size <
            0x21,            // block header: raw block and last one
            0x0,
            0x0,  // <
            0x10, // block content
            0x20,
            0x30,
            0x40, //<
            0x01, // content checksum of value 1
            0x00,
            0x00,
            0x00, //<
            0x42, // additionnal byte
        ]);

        let res = frame::ZStandardFrame::parse(&mut parser).unwrap();

        assert_eq!(Some(1), res.checksum());
        assert_eq!(1, res.blocks().len());
        assert_eq!(1, parser.len());
    }

    #[test]
    fn parse_without_checksum_ok() {
        let mut parser = ForwardByteParser::new(&[
            0b01_1_0_0_0_00, // single segment, so FCS is 1 byte, w/o checksum
            0x04,            // frame content size <
            0x21,            // block header: raw block and last one
            0x0,
            0x0,  // <
            0x10, // block content
            0x20,
            0x30,
            0x40, //<
            0x42, // additionnal byte
        ]);

        let res = frame::ZStandardFrame::parse(&mut parser).unwrap();

        assert_eq!(None, res.checksum());
        assert_eq!(1, res.blocks().len());
        assert_eq!(1, parser.len());
    }

    #[test]
    fn parse_no_checksum_error() {
        let mut parser = ForwardByteParser::new(&[
            0b01_1_0_0_1_00, // single segment, so FCS is 1 byte, w/ checksum
            0x04,            // frame content size <
            0x21,            // block header: raw block and last one
            0x0,
            0x0,  // <
            0x10, // block content
            0x20,
            0x30,
            0x40,
            0x42, //< // no checksum at the end, should throw error
        ]);

        let res = frame::ZStandardFrame::parse(&mut parser);

        assert!(matches!(
            res,
            Err(frame::Error::MissingChecksum(
                parsing::Error::NotEnoughBytes {
                    requested: 4,
                    available: 1
                }
            ))
        ))
    }

    #[test]
    fn parse_window_size_too_big_error() {
        let mut parser = ForwardByteParser::new(&[
            0b01_0_0_0_1_00, // not single segment, so FCS is 2 bytes and there is a window desc, w/ checksum
            0xff,            // max size window
            0x04,            // frame content size
            0x05,            // <
            0x21,            // block header: raw block and last one
            0x0,
            0x0,  // <
            0x10, // block content
            0x20,
            0x30,
            0x40,
            0x42, //< // no checksum at the end, should throw error
        ]);

        let res = frame::ZStandardFrame::parse(&mut parser);

        let _got = (1u64 << 41) + 7 * (1u64 << 38);
        assert!(matches!(
            res,
            Err(frame::Error::WindowSizeTooBig {
                max: frame::MAX_WIN_SIZE,
                got: _got
            })
        ))
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
