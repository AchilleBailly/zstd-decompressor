
mod huffman_test {
    use zstd_decompressor::decoders::huffman::decoders::huffman;
    use huffman::HuffmanDecoder;

    #[test]
    fn example_tree() {
        let example = huffman::build_example_tree();
        println!("{:?}", example); //should be smething like : HuffmanDecoder { 00: 65, 01: 67, 1: 66 }
        let widths: Vec<_> = std::iter::repeat(0).take(65).chain([2, 1, 2]).collect();
        println!("{:?}", HuffmanDecoder::from_number_of_bits(widths));
        let weights: Vec<_> = std::iter::repeat(0).take(65).chain([1, 2]).collect();
        println!("{:?}", HuffmanDecoder::from_weights(weights));
    }
}