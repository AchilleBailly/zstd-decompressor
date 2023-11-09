#[cfg(test)]
mod block_tests {
    use zstd_decompressor::block::{self, Block};
    use zstd_decompressor::parsing::{self, ForwardByteParser};

    #[test]
    fn decode_raw_block_last() {
        let mut parser = ForwardByteParser::new(&[
            // Raw block, last block, len 4, content 0x10, 0x20, 0x30, 0x40,
            // and an extra 0x50 at the end.
            0x21, 0x0, 0x0, 0x10, 0x20, 0x30, 0x40, 0x50,
        ]);
        let (block, last) = Block::parse(&mut parser).unwrap();
        assert!(last);
        assert!(matches!(block, Block::RawBlock(&[0x10, 0x20, 0x30, 0x40])));
        assert_eq!(1, parser.len());
        let decoded = block.decode().unwrap();
        assert_eq!(vec![0x10, 0x20, 0x30, 0x40], decoded);
    }

    #[test]
    fn decode_rle_block_not_last() {
        let mut parser = ForwardByteParser::new(&[
            // RLE block, not last, byte 0x42 and repeat 0x30004,
            // and an extra 0x50 at the end.
            0x22, 0x0, 0x18, 0x42, 0x50,
        ]);
        let (block, last) = Block::parse(&mut parser).unwrap();
        assert!(!last);
        assert!(matches!(
            block,
            Block::RLEBlock {
                byte: 0x42,
                repeat: 196612
            }
        ));
        assert_eq!(1, parser.len());
        let decoded = block.decode().unwrap();
        assert_eq!(196612, decoded.len());
        assert!(decoded.into_iter().all(|b| b == 0x42));
    }

    #[test]
    fn reserved_block_error_test() {
        let mut parser = ForwardByteParser::new(&[
            // Reserved block, last block, len 4, content 0x10, 0x20, 0x30, 0x40,
            // and an extra 0x50 at the end.
            0x27, 0x0, 0x0, 0x10, 0x20, 0x30, 0x40, 0x50,
        ]);

        let res = Block::parse(&mut parser);
        assert!(matches!(res, Err(block::Error::ReservedBlockType())));
    }

    #[test]
    fn not_enough_bytes_error_test() {
        let mut parser = ForwardByteParser::new(&[
            // Raw block, last block, len 4, content 0x10, 0x20, 0x30, 0x40,
            // and an extra 0x50 at the end.
            0x21, 0x0, 0x0, 0x10, 0x20, 0x30,
        ]);

        let res = Block::parse(&mut parser);
        assert!(matches!(
            res,
            Err(block::Error::ParsingError(parsing::Error::NotEnoughBytes {
                requested: 4,
                available: 3
            }))
        ));
    }
}
