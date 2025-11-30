#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]
mod TransformationMethods;
mod BitStream;
mod LZWCoderEnhanced;
mod Huffman;

use std::{fs, result, time::{Duration, Instant}};
use crate::TransformationMethods::{BWT, inverse_transform_file, transform_file};

fn encode_file_with_timer(input_path: String, output_path: String, encoding_type: String, transform_id: u8) {
    println!("Encoding file (Type: {encoding_type}; transform id: {transform_id}): {input_path}");
    let start = Instant::now();

    let encoding_handle = if encoding_type == "LZW" {
        std::thread::spawn(move || {
            LZWCoderEnhanced::encode_file(&input_path, &output_path, true, transform_id);
            start.elapsed()
        })
    } else if encoding_type == "Huffman" {
        std::thread::spawn(move || {
            Huffman::HuffmanEncoder::encode(&input_path, &output_path, transform_id);
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

fn decode_file_with_timer(input_path: String, output_path: String, encoding_type: String, transform_id: u8) {
    println!("Decoding file (Type: {encoding_type}; transform id: {transform_id}): {}", input_path);
    let start = Instant::now();
    
    let decoding_handle = if encoding_type == "LZW" {
        std::thread::spawn(move || {
            LZWCoderEnhanced::decode_file(&input_path, &output_path, transform_id);
            start.elapsed()
        })
    } else if encoding_type == "Huffman" {
        std::thread::spawn(move || {
            Huffman::HuffmanDecoder::decode(&input_path, &output_path, transform_id);
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

fn generate_args_and_paths(filenames: &Vec<&str>, base_input: &str, base_output_encoded: &str, base_output_decoded: &str, encoding_type: &str) 
    -> Vec<(String, String, String, String, u8)> {
    
    let mut results = Vec::new();

    for filename in filenames {
        let no_suff = filename.split('.').next().unwrap();
        let f_type = filename.split('.').last().unwrap();

        fs::create_dir_all(&format!("{base_output_encoded}/{no_suff}")).unwrap();
        fs::create_dir_all(&format!("{base_output_decoded}/{no_suff}")).unwrap();

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.huff");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_huff.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "Huffman".to_string(), 0));

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.hufft_comb");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_hufft.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "Huffman".to_string(), 1));

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.hufft_bwt");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_hufft.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "Huffman".to_string(), 2));

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.hufft_mft");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_hufft.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "Huffman".to_string(), 3));

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.lzw");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_lzw.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "LZW".to_string(), 0));

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.lzwt_comb");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_lzw.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "LZW".to_string(), 1));

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.lzwt_bwt");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_lzw.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "LZW".to_string(), 2));

        let input_path = format!("{}/{}", base_input, filename);
        let output_path_encoded = format!("{base_output_encoded}/{no_suff}/{filename}.lzwt_mft");
        let output_path_decoded = format!("{base_output_decoded}/{no_suff}/decoded_{no_suff}_lzwt.{f_type}");

        results.push((input_path, output_path_encoded, output_path_decoded, "LZW".to_string(), 3));
    }

    return results;
}

fn main() {
    let filenames = vec![
        // "file_1.txt",
        "file_2.txt",
        "file_3.pdf",
        "file_4.pdf",
        "file_5.msi",
        "file_6.jpg",
        "file_7.exe",
        "file_8.bin",
        "file_9.csv",
    ];

    let args = generate_args_and_paths(
        &filenames,
        "test_data/input",
        "test_data/output/encoded",
        "test_data/output/decoded",
        "",
    );

    for (input_path, output_path_encoded, output_path_decoded, encoding_type, use_transform) in args {
        encode_file_with_timer(
            input_path.clone(),
            output_path_encoded.clone(),
            encoding_type.clone(),
            use_transform,
        );

        // decode_file_with_timer(
        //     output_path_encoded.clone(),
        //     output_path_decoded.clone(),
        //     encoding_type.clone(),
        //     use_transform,
        // );
    }
}
