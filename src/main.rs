#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]
mod TransformationMethods;
mod BitStream;
mod LZWCoderEnhanced;
mod Huffman;

use std::time::{Instant, Duration};
use crate::TransformationMethods::{BWT, inverse_transform_file, transform_file};

fn encode_file_with_timer(input_path: String, output_path: String, encoding_type: String, use_transform: bool) {
    println!("Encoding file (Type: {encoding_type}; use transform: {use_transform}): {input_path}");
    let start = Instant::now();

    let encoding_handle = if encoding_type == "LZW" {
        std::thread::spawn(move || {
            LZWCoderEnhanced::encode_file(&input_path, &output_path, true, use_transform);
            start.elapsed()
        })
    } else if encoding_type == "Huffman" {
        std::thread::spawn(move || {
            Huffman::HuffmanEncoder::encode(&input_path, &output_path, use_transform);
            start.elapsed()
        })
    } else {
        println!("Unknown encoding type: {}", encoding_type);
        return;
    };
    
    // Progress time
    while !encoding_handle.is_finished() {
        print!("\rEncoding time: {:?}", start.elapsed());
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::thread::sleep(Duration::from_millis(100));
    }
    
    let encode_duration = encoding_handle.join().unwrap();
    println!("\rEncoding time: {:?}", encode_duration);
}

fn decode_file_with_timer(input_path: String, output_path: String, encoding_type: String, use_transform: bool) {
    println!("Decoding file (Type: {encoding_type}; use transform: {use_transform}): {}", input_path);
    let start = Instant::now();
    
    let decoding_handle = if encoding_type == "LZW" {
        std::thread::spawn(move || {
            LZWCoderEnhanced::decode_file(&input_path, &output_path, use_transform);
            start.elapsed()
        })
    } else if encoding_type == "Huffman" {
        std::thread::spawn(move || {
            Huffman::HuffmanDecoder::decode(&input_path, &output_path, use_transform);
            start.elapsed()
        })
    } else {
        println!("Unknown encoding type: {}", encoding_type);
        return;
    };

    // Progress time
    while !decoding_handle.is_finished() {
        print!("\rDecoding time: {:?}", start.elapsed());
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    let decode_duration = decoding_handle.join().unwrap();
    println!("\rDecoding time: {:?}", decode_duration);
}

fn main() {
    // encode_file("test_data/4.txt", "test_data/4.txt.lzwe", true);
    // decode_file("test_data/4.txt.lzwe", "test_data/decoded_4.txt");

    // Huffman::HuffmanEncoder::encode("test_data/test_pdf_2.pdf", "test_data/test_pdf_2_1.pdf.huff");
    // Huffman::HuffmanDecoder::decode("test_data/test_pdf_2_1.pdf.huff", "test_data/decoded_pdf_2_1.pdf");

    // transform_file("test_data/test_pdf.pdf", "test_data/test_pdf.pdf.transformed");
    // inverse_transform_file("test_data/test_pdf.pdf.transformed", "test_data/inversed.pdf");

    encode_file_with_timer(
        "test_data/input/file_3.pdf".to_string(),
        "test_data/output/encoded/file_3.pdf.huff".to_string(),
        "Huffman".to_string(),
        true,
    );

    decode_file_with_timer(
        "test_data/output/encoded/file_3.pdf.huff".to_string(),
        "test_data/output/decoded/file_3_huff.pdf".to_string(),
        "Huffman".to_string(),
        true,
    );

    encode_file_with_timer(
        "test_data/input/file_3.pdf".to_string(),
        "test_data/output/encoded/file_3.pdf.lzwe".to_string(),
        "LZW".to_string(),
        true,
    );

    decode_file_with_timer(
        "test_data/output/encoded/file_3.pdf.lzwe".to_string(),
        "test_data/output/decoded/file_3_lzwe.pdf".to_string(),
        "LZW".to_string(),
        true,
    );
}
