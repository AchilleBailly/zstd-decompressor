#[cfg(test)]
mod sequence_decoer_tests {
    use zstd_decompressor::parsing::ForwardByteParser;

    #[test]
    fn fuzzer_panic_ok() {
        let data = [
            40, 181, 47, 253, 0, 10, 165, 0, 0, 85, 47, 0, 252, 59, 64, 44, 0, 51, 29, 44, 47, 10,
            40, 0, 181, 181, 40, 181, 47, 253,
        ];

        let parser = ForwardByteParser::new(&data);
        for frame in parser.iter() {
            match frame {
                Ok(zstd_decompressor::frame::Frame::SkippableFrame(skippable)) => {
                    // if args.print_skippable {
                    //     res.push(
                    //         &mut zstd_decompressor::frame::Frame::SkippableFrame(skippable).decode(),
                    //     )
                    // }
                }
                Ok(frame) => {
                    frame.decode();
                }
                Err(e) => {
                    dbg!(e);
                    break;
                }
            }
        }
    }
}
