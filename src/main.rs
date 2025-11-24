#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]
mod TransformationMethods;
mod BitStream;
mod LZWCoderEnhanced;
mod Huffman;

use crate::TransformationMethods::BWT;

use crate::LZWCoderEnhanced::{encode_file, decode_file};

fn main() {
    encode_file("test_data/4.txt", "test_data/4.txt.lzwe", true);
    decode_file("test_data/4.txt.lzwe", "test_data/decoded_4.txt");
}
