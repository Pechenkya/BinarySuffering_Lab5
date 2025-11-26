const MAX_DICT_SIZE: usize = 0xFFFF;
const CLEAR_SYMBOL: u16 = 0xFFFF;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::collections::HashMap;

use crate::TransformationMethods::*;

struct LZWCoderEnhanced {
    dict: Vec<(u8, Option<u16>)>,
    reverse_dict_map: HashMap<(u8, Option<u16>), u16>,  // Used for O(1) lookup for elements, doubles memory usage
    max_dict_size: usize,
    clear_dict_on_overfill: bool
}

impl LZWCoderEnhanced {
    fn set_init_dict(&mut self) {
        self.dict.clear();
        self.reverse_dict_map.clear();
        
        for i in 0..256 {
            self.dict.push((i as u8, None));
            self.reverse_dict_map.insert((i as u8, None), i as u16);
        }
    }

    fn find_seq_in_dict(&self, (char, idx): (u8, Option<u16>)) -> Option<u16> {
        if let Some(&res_idx) = self.reverse_dict_map.get(&(char, idx)) {
            Some(res_idx)
        } else {
            None
        }
    }

    // Returns true if added, false if not added (dict full)
    fn add_seq_to_dict(&mut self, (char, idx): (u8, Option<u16>)) -> bool {
        if self.dict.len() < self.max_dict_size {
            self.dict.push((char, idx));
            self.reverse_dict_map.insert((char, idx), self.get_last_dict_index());
            
            return true;
        } else {
            return false;
        }
    }

    fn recover_seq_from_dict(&self, mut idx: u16) -> Option<Vec<u8>> {
        let mut seq: Vec<u8> = Vec::new();

        while let Some((char, next_idx)) = self.dict.get(idx as usize) {
            seq.push(*char);
            if let Some(next_idx) = next_idx {
                idx = *next_idx;
            } else {
                break;
            }
        }

        if seq.is_empty() {
            None
        } else {
            seq.reverse();
            Some(seq)
        }
    }

    fn get_last_dict_index(&self) -> u16 {
        (self.dict.len() - 1) as u16
    }
}

pub fn encode(input: &[u8], clear_dict_on_overfill: bool) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();

    // Create encoder and initialize dictionary
    let mut internal_encoder = LZWCoderEnhanced {
        dict: Vec::new(),
        reverse_dict_map: HashMap::new(),
        max_dict_size: MAX_DICT_SIZE,
        clear_dict_on_overfill
    };

    // Store parameters for decoder into first three bytes
    output.push(if clear_dict_on_overfill { 1 } else { 0 });
    output.extend_from_slice(&(internal_encoder.max_dict_size as u16).to_le_bytes());

    internal_encoder.set_init_dict();

    let mut I: Option<u16> = None;

    for slice in input.chunks(BWT_BLOCK_SIZE) {
        for &byte in perform_BWT_MTF(&slice.to_vec()).iter() {
            if let Some(idx) = internal_encoder.find_seq_in_dict((byte, I)) {
                I = Some(idx);
            } else {
                output.extend_from_slice(&I.unwrap().to_le_bytes());

                let pair_added = internal_encoder.add_seq_to_dict((byte, I));

                if !pair_added && internal_encoder.clear_dict_on_overfill {
                    internal_encoder.set_init_dict();
                    output.extend_from_slice(&CLEAR_SYMBOL.to_le_bytes());
                }

                I = Some(byte as u16);  // I -> idx of byte (bytes are filled sequentially)
            }
        }
    }

    output.extend_from_slice(&I.unwrap().to_le_bytes());

    return output;
}

pub fn decode(input: &[u8]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();

    // Read first three bytes to restore parameters of encoder
    let clear_dict_on_overfill = input[0] != 0;
    let last_dict_index = u16::from_le_bytes(input[1..3].try_into().unwrap());

    // Create decoder and initialize dictionary
    let mut internal_decoder = LZWCoderEnhanced {
        dict: Vec::new(),
        reverse_dict_map: HashMap::new(),
        max_dict_size: last_dict_index as usize + 1,    // We store only two bytes to ensure the limitation of max 16 bits for code
        clear_dict_on_overfill
    };
    internal_decoder.set_init_dict();

    // Read first idx
    let I = u16::from_le_bytes(input[3..5].try_into().unwrap());

    // First byte should be always in the dict
    if let Some((fb, _)) = internal_decoder.dict.get(I as usize) {
        output.push(*fb);   // Send it directly to output
    } else {
        panic!("Corrupted input data: first index not in dictionary");
    }

    let mut old_I: u16 = I;
    let mut is_first = true;

    for chunk in input[5..].chunks(2) {
        // Read next idx
        let I = u16::from_le_bytes(chunk.try_into().unwrap());

        // First byte logic
        if is_first {
            is_first = false;
            // First byte should be always in the dict
            if let Some((fb, _)) = internal_decoder.dict.get(I as usize) {
                output.push(*fb);   // Send it directly to output
            } else {
                panic!("Corrupted input data: first index not in dictionary");
            }

            old_I = I;
            continue;
        }

        // Check clear symbol
        if I == CLEAR_SYMBOL {
            internal_decoder.set_init_dict();
            is_first = true;
            continue;
        }

        // Normal processing
        if let Some(S) = internal_decoder.recover_seq_from_dict(I) {
            output.extend_from_slice(&S);
            internal_decoder.add_seq_to_dict((S[0], Some(old_I)));
            old_I = I;
        } else {
            // Special case (only case when I is not in dict - covering sequences)
            // S = old_S || old_S[0]
            if let Some(old_S) = internal_decoder.recover_seq_from_dict(old_I) {
                output.extend_from_slice(&old_S);
                output.push(old_S[0]);

                // Add this sequence to the dict
                internal_decoder.add_seq_to_dict((old_S[0], Some(old_I)));

                // Set I to newly added sequence
                old_I = internal_decoder.get_last_dict_index();
            }
        }
    }

    // Detransform BWT+MTF
    output = output.chunks(BWT_RESULT_SIZE).map(|chunk| perform_inverse_MTF_BWT(&chunk.to_vec())).flatten().collect();

    return output;
}

pub fn encode_file(input_path: &str, output_path: &str, clear_dict_on_overfill: bool) {
    let input_file = File::open(input_path).unwrap();
    let mut reader = BufReader::new(input_file);

    let output_file = OpenOptions::new().write(true)
                                        .create(true)
                                        .truncate(true)
                                        .open(output_path).unwrap();
    let mut writer = BufWriter::new(output_file);

    // Create encoder and initialize dictionary
    let mut internal_encoder = LZWCoderEnhanced {
        dict: Vec::new(),
        reverse_dict_map: HashMap::new(),
        max_dict_size: MAX_DICT_SIZE,
        clear_dict_on_overfill
    };

    // Store parameters for decoder into first three bytes
    writer.write(&[ if clear_dict_on_overfill { 1 } else { 0 } ]).unwrap();
    writer.write(&(internal_encoder.max_dict_size as u16).to_le_bytes()).unwrap();

    internal_encoder.set_init_dict();

    let mut I: Option<u16> = None;

    let mut slice: Vec<u8> = Vec::with_capacity(BWT_BLOCK_SIZE);
    slice.resize(BWT_BLOCK_SIZE, 0);

    while let Ok(_bytes_read) = reader.read(&mut slice) {   // Buffered read for transformation
        if _bytes_read == 0 {
            break;  // EOF
        }

        slice.truncate(_bytes_read);

        for &byte in perform_BWT_MTF(&slice.to_vec()).iter() {
            if let Some(idx) = internal_encoder.find_seq_in_dict((byte, I)) {
                I = Some(idx);
            } else {
                writer.write(&I.unwrap().to_le_bytes()).unwrap();

                let pair_added = internal_encoder.add_seq_to_dict((byte, I));

                if !pair_added && internal_encoder.clear_dict_on_overfill {
                    internal_encoder.set_init_dict();
                    writer.write(&CLEAR_SYMBOL.to_le_bytes()).unwrap();
                }

                I = Some(byte as u16);  // I -> idx of byte (bytes are filled sequentially)
            }
        }
    }

    writer.write(&I.unwrap().to_le_bytes()).unwrap();
}

pub fn decode_file(input_path: &str, output_path: &str) {
    let input_file = File::open(input_path).unwrap();
    let mut reader = BufReader::new(input_file);

    let output_file = OpenOptions::new().write(true)
                                        .create(true)
                                        .truncate(true)
                                        .open(output_path).unwrap();
    let mut writer = BufWriter::new(output_file);

    // Read first three bytes to restore parameters of encoder
    let mut param_buff = [0u8; 3];
    reader.read_exact(&mut param_buff).unwrap();
    let clear_dict_on_overfill = param_buff[0] != 0;
    let last_dict_index = u16::from_le_bytes(param_buff[1..3].try_into().unwrap());

    // Create decoder and initialize dictionary
    let mut internal_decoder = LZWCoderEnhanced {
        dict: Vec::new(),
        reverse_dict_map: HashMap::new(),
        max_dict_size: last_dict_index as usize + 1,    // We store only two bytes to ensure the limitation of max 16 bits for code
        clear_dict_on_overfill,
    };
    internal_decoder.set_init_dict();

    let mut idx_buff = [0u8; 2];
    let mut is_first = true;
    let mut old_I = 0;

    let mut transormation_slice: Vec<u8> = Vec::new();

    while let Some(_) = reader.read_exact(&mut idx_buff).ok() {
        // Read next idx
        let I = u16::from_le_bytes(idx_buff.try_into().unwrap());

        // First byte logic
        if is_first {
            is_first = false;
            // First byte should be always in the dict
            if let Some((fb, _)) = internal_decoder.dict.get(I as usize) {
                // writer.write(&[*fb]).unwrap();   // Send it directly to output
                transormation_slice.push(*fb);
            } else {
                panic!("Corrupted input data: first index not in dictionary");
            }

            old_I = I;
            continue;
        }

        // Check clear symbol
        if I == CLEAR_SYMBOL {
            internal_decoder.set_init_dict();
            is_first = true;
            continue;
        }

        // Normal processing
        if let Some(S) = internal_decoder.recover_seq_from_dict(I) {
            // writer.write(&S).unwrap();
            transormation_slice.extend_from_slice(&S);

            internal_decoder.add_seq_to_dict((S[0], Some(old_I)));
            old_I = I;
        } else {
            // Special case (only case when I is not in dict - covering sequences)
            // S = old_S || old_S[0]
            if let Some(old_S) = internal_decoder.recover_seq_from_dict(old_I) {
                // writer.write(&old_S).unwrap();
                // writer.write(&old_S[0..1]).unwrap();
                transormation_slice.extend_from_slice(&old_S);
                transormation_slice.push(old_S[0]);

                // Add this sequence to the dict
                internal_decoder.add_seq_to_dict((old_S[0], Some(old_I)));

                // Set I to newly added sequence
                old_I = internal_decoder.get_last_dict_index();
            }
        }

        // Flush transformation slice if it reached BWT_RESULT_SIZE
        while transormation_slice.len() >= BWT_RESULT_SIZE {
            let to_process = &transormation_slice.drain(0..BWT_RESULT_SIZE).collect();
            let detransformed = perform_inverse_MTF_BWT(to_process);
            writer.write(&detransformed).unwrap();
        }
    }

    // Flush remaining transformation
    if transormation_slice.len() > 0 {
        writer.write(&perform_inverse_MTF_BWT(&transormation_slice)).unwrap();
    }
}
