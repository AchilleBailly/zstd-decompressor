# NET7212 Achille Bailly and Clément Naves

This repo contains our code for a ZSTD decompressor built in Rust.

## Usage

This decompressor is aimed at ZSTD compressed text file and can be ran with the following command:  
`cargo run -- compressed_file_path`  
You can also print info about the frames contained in the file with the `--info` option:  
`cargo run -- --info compressed_file_path`  
  
By default, the program will output the decompressed file to stdout, you can choose an output with `-o <filename>` or `--output <filename>` option. This will overwrite the content of the given file.  
  
By default, Skippable frame and not decoded or printed, you can include them with the `--print-skippable` option.  
Don't forget you can also print the help with `cargo run -- --help`.  

## What was done
The decompressor fully works. Unit tests and some integration tests were written but time took over at some point and we had to focus on functionnal code. It may had lead to uneventful bugs that ruined the whole decompressor. For example :
 - In `DecodingContext::decode_offset`, we mixed the array indices with the RFC notation starting at 1 and returned offset2 (`offsets[1]`) instead of offset1 (`offsets[0]`). While it did not crash the program, it lead to a very mixed up output.
 - In `LiteralsSeciont::parse_header`, in the match arm `-> compressed/treeless -> 2`, we told the parser to take 8 bits instead of the current 10. While the returned value did not change anything since we do not use it, it meant that the next value we read from the bitstream was shifted by 2 bits in said bitstream. That one on the other hand, was particularly important to know wher to stop the literals section.


While the tests were not fully done, we used a fuzzer (very basic but nonetheless...) that helped us correct edge cases to prevent the program from panicking unexpectedly. 