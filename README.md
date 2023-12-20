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
The decompressor works. At least it works for the compressed file `romeo.txt.zst` but when trying with `moby-dick.txt` compressed, it seems to run forever, or at least be very very slow. We'll see if we can improve on that or find the bug.   