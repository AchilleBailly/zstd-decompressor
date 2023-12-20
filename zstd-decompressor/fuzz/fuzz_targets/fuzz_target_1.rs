#![no_main]
#[macro_use]
extern crate libfuzzer_sys;

extern crate zstd_decompressor;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    println!("New data, len: {}", data.len());
    let parser = zstd_decompressor::parsing::ForwardByteParser::new(data);

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
                &mut frame.decode();
            }
            Err(e) => {
                dbg!(e);
                break;
            }
        }
    }
});
