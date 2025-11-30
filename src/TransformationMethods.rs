use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::{result, usize};

// pub const TRANSFORM_BLOCK_SIZE: usize = 256;
// pub const BWT_RESULT_SIZE: usize = TRANSFORM_BLOCK_SIZE + 1;

pub const TRANSFORM_BLOCK_SIZE: usize = 4096;
pub const BWT_RESULT_SIZE: usize = TRANSFORM_BLOCK_SIZE + 2;


fn generate_shifts(input_string: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut shifts = Vec::new();

    let length = input_string.len();
    for i in 0..length {
        shifts.push([&input_string[i..], &input_string[..i]].concat().to_vec());
    }

    shifts
}

pub fn BWT(input_string: &Vec<u8>) -> Vec<u8> {
    // Limited to TRANSFORM_BLOCK_SIZE bytes
    if input_string.len() > TRANSFORM_BLOCK_SIZE {
        panic!("BWT can only handle inputs of size {} (passed: {})", TRANSFORM_BLOCK_SIZE, input_string.len());
    }

    let mut shifts: Vec<Vec<u8>> = generate_shifts(&input_string);
    shifts.sort_by_key(|x| x.clone());

    let mut bwt_result = Vec::new();
    let mut original_index: u16 = 0;
    for (id, shift) in shifts.iter().enumerate() {
        bwt_result.push(*shift.last().unwrap());
        if shift == input_string {
            original_index = id as u16;
        }
    }

    // Save the original index byte
    if TRANSFORM_BLOCK_SIZE <= 256 {
        bwt_result.push(original_index as u8);
    } else {
        bwt_result.extend_from_slice(&original_index.to_le_bytes());
    }

    bwt_result
}

pub fn inverse_BWT(bwt_string: &Vec<u8>) -> Vec<u8> {
    // Limited to BWT_RESULT_SIZE bytes input (BWT_RESULT_SIZE + BWT_RESULT_SIZE // 8 for original index)
    if bwt_string.len() > BWT_RESULT_SIZE {
        panic!("BWT inverse can only handle inputs of size {} (passed: {})", BWT_RESULT_SIZE, bwt_string.len());
    }

    let mut length = bwt_string.len() - 1;
    let mut pos = bwt_string[length] as usize; // Last byte is the original index

    if TRANSFORM_BLOCK_SIZE > 256 {
        length = length - 1;
        pos = u16::from_le_bytes(bwt_string[length..].try_into().unwrap()) as usize;
    }

    let mut enumerated = bwt_string[..length].into_iter().enumerate().collect::<Vec<(usize, &u8)>>();
    enumerated.sort_by_key(|&(_, &byte)| byte);

    let table: Vec<usize> = enumerated.iter().map(|&(idx, _)| idx).collect();

    let mut result = Vec::with_capacity(length);

    for _ in 0..length {
        pos = table[pos];
        result.push(bwt_string[pos]);
    }

    result
}

pub fn MTF(input_string: &Vec<u8>) -> Vec<u8> {
    let mut symbol_table: Vec<u8> = (0..=255).collect();
    let mut mtf_result = Vec::new();

    for &byte in input_string.iter() {
        let index = symbol_table.iter().position(|&b| b == byte).unwrap();
        mtf_result.push(index as u8);

        // Move the accessed byte to the front
        symbol_table.remove(index);
        symbol_table.insert(0, byte);
    }

    mtf_result
}

pub fn inverse_MTF(mtf_string: &Vec<u8>) -> Vec<u8> {
    let mut symbol_table: Vec<u8> = (0..=255).collect();
    let mut result = Vec::new();

    for &index in mtf_string.iter() {
        let byte = symbol_table.remove(index as usize);
        result.push(byte);

        // Move the accessed byte to the front
        symbol_table.insert(0, byte);
    }
    
    result
}

pub fn perform_BWT_MTF(input_string: &Vec<u8>) -> Vec<u8> {
    let bwt_result = BWT(input_string);
    let mtf_result = MTF(&bwt_result);
    mtf_result
}

pub fn perform_inverse_MTF_BWT(mtf_string: &Vec<u8>) -> Vec<u8> {
    let inverse_mtf_result = inverse_MTF(mtf_string);
    let inverse_bwt_result = inverse_BWT(&inverse_mtf_result);
    inverse_bwt_result
}

pub fn perform_transform(input_string: &Vec<u8>, transform_id: u8) -> Vec<u8> {
    if transform_id == 1 {    // Both BWT and MTF
        return perform_BWT_MTF(input_string);
    } else if transform_id == 2 {   // Only BWT
        return BWT(input_string)
    } else if transform_id == 3 {   // Only MTF
        return MTF(input_string)
    } else {
        panic!("Unknown transform: {}", transform_id);
    }
}

pub fn perform_inverse_transform(input_string: &Vec<u8>, transform_id: u8) -> Vec<u8> {
    if transform_id == 1 {    // Both BWT and MTF
        return perform_inverse_MTF_BWT(input_string);
    } else if transform_id == 2 {   // Only BWT
        return inverse_BWT(input_string)
    } else if transform_id == 3 {   // Only MTF
        return inverse_MTF(input_string)
    } else {
        panic!("Unknown inverse transform: {}", transform_id);
    }
}

pub fn transform_file(input_path: &str, output_path: &str, transform_id: u8) {
    let mut input_file = BufReader::new(File::open(input_path).expect("Failed to open input file"));
    let mut output_file = BufWriter::new(File::create(output_path).expect("Failed to create output file"));

    let mut buffer = Vec::new();
    let mut slice: Vec<u8> = Vec::with_capacity(TRANSFORM_BLOCK_SIZE);
    slice.resize(TRANSFORM_BLOCK_SIZE, 0);

    while let Ok(_bytes_read) = input_file.read(&mut slice) {   // Buffered read for transformation
        if _bytes_read == 0 {
            break;  // EOF
        }

        buffer.extend_from_slice(&slice[.._bytes_read]);

        if buffer.len() >= TRANSFORM_BLOCK_SIZE {
            let block: Vec<u8> = buffer.drain(0..TRANSFORM_BLOCK_SIZE).collect();
            
            let result: Vec<_> = perform_transform(&block, transform_id);
            output_file.write_all(&result).expect("Failed to write transformed data");
        }
    }

    if buffer.len() > 0 {
        let result: Vec<_> = perform_BWT_MTF(&buffer);
        output_file.write_all(&result).expect("Failed to write transformed data");
    }
}

pub fn inverse_transform_file(input_path: &str, output_path: &str, transform_id: u8) {
    let mut input_file = BufReader::new(File::open(input_path).expect("Failed to open input file"));
    let mut output_file = BufWriter::new(File::create(output_path).expect("Failed to create output file"));

    let mut buffer = Vec::new();
    
    let transform_block_size = if transform_id == 1 {
        BWT_RESULT_SIZE
    } else {
        TRANSFORM_BLOCK_SIZE
    };

    let mut slice: Vec<u8> = Vec::with_capacity(transform_block_size);
    slice.resize(transform_block_size, 0);

    while let Ok(_bytes_read) = input_file.read(&mut slice) {   // Buffered read for inverse transformation
        if _bytes_read == 0 {
            break;  // EOF
        }

        buffer.extend_from_slice(&slice[.._bytes_read]);

        while buffer.len() >= transform_block_size {
            let block: Vec<u8> = buffer.drain(0..transform_block_size).collect();
            let detransformed = perform_inverse_transform(&block, transform_id);
            output_file.write_all(&detransformed).expect("Failed to write inversed data");
        }
    }

    if buffer.len() > 0 {
        let detransformed = perform_inverse_MTF_BWT(&buffer);
        output_file.write_all(&detransformed).expect("Failed to write inversed data");
    }
}