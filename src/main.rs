#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]
mod TransformationMethods;
mod BitStream;
mod LZWCoderEnhanced;
mod Huffman;

use crate::TransformationMethods::{BWT, inverse_transform_file, transform_file};

use crate::LZWCoderEnhanced::{encode_file, decode_file};

fn main() {
    // encode_file("test_data/4.txt", "test_data/4.txt.lzwe", true);
    // decode_file("test_data/4.txt.lzwe", "test_data/decoded_4.txt");

    Huffman::HuffmanEncoder::encode("test_data/test_pdf_2.pdf", "test_data/test_pdf_2_1.pdf.huff");
    Huffman::HuffmanDecoder::decode("test_data/test_pdf_2_1.pdf.huff", "test_data/decoded_pdf_2_1.pdf");

    // transform_file("test_data/test_pdf.pdf", "test_data/test_pdf.pdf.transformed");
    // inverse_transform_file("test_data/test_pdf.pdf.transformed", "test_data/inversed.pdf");
}
