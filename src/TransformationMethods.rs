// pub const BWT_BLOCK_SIZE: usize = 256;
// pub const BWT_RESULT_SIZE: usize = BWT_BLOCK_SIZE + 1;

pub const BWT_BLOCK_SIZE: usize = 2048;
pub const BWT_RESULT_SIZE: usize = BWT_BLOCK_SIZE + 2;


fn generate_shifts(input_string: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut shifts = Vec::new();

    let length = input_string.len();
    for i in 0..length {
        shifts.push([&input_string[i..], &input_string[..i]].concat().to_vec());
    }

    shifts
}

pub fn BWT(input_string: Vec<u8>) -> Vec<u8> {
    // Limited to BWT_BLOCK_SIZE bytes
    if input_string.len() > BWT_BLOCK_SIZE {
        panic!("BWT can only handle inputs of size {} (passed: {})", BWT_BLOCK_SIZE, input_string.len());
    }

    let mut shifts: Vec<Vec<u8>> = generate_shifts(&input_string);
    shifts.sort_by_key(|x| x.clone());

    let mut bwt_result = Vec::new();
    let mut original_index: u16 = 0;
    for (id, shift) in shifts.iter().enumerate() {
        bwt_result.push(*shift.last().unwrap());
        if shift == &input_string {
            original_index = id as u16;
        }
    }

    // Save the original index byte
    if BWT_BLOCK_SIZE <= 256 {
        bwt_result.push(original_index as u8);
    } else {
        bwt_result.extend_from_slice(&original_index.to_le_bytes());
    }
    
    bwt_result
}

pub fn inverse_BWT(bwt_string: Vec<u8>) -> Vec<u8> {
    // Limited to BWT_RESULT_SIZE bytes input (BWT_RESULT_SIZE + BWT_RESULT_SIZE // 8 for original index)
    if bwt_string.len() > BWT_RESULT_SIZE {
        panic!("BWT inverse can only handle inputs of size {} (passed: {})", BWT_RESULT_SIZE, bwt_string.len());
    }

    let mut length = bwt_string.len() - 1;
    let mut pos = bwt_string[length] as usize; // Last byte is the original index

    if BWT_BLOCK_SIZE > 256 {
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

pub fn MTF(input_string: Vec<u8>) -> Vec<u8> {
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

pub fn inverse_MTF(mtf_string: Vec<u8>) -> Vec<u8> {
    let mut symbol_table: Vec<u8> = (0..=255).collect();
    let mut result = Vec::new();

    for &index in mtf_string.iter() {
        let byte = symbol_table[index as usize];
        result.push(byte);

        // Move the accessed byte to the front
        symbol_table.remove(index as usize);
        symbol_table.insert(0, byte);
    }

    result
}

pub fn perform_BWT_MTF(input_string: Vec<u8>) -> Vec<u8> {
    let bwt_result = BWT(input_string);
    let mtf_result = MTF(bwt_result);
    mtf_result
}

pub fn perform_inverse_MTF_BWT(mtf_string: Vec<u8>) -> Vec<u8> {
    let inverse_mtf_result = inverse_MTF(mtf_string);
    let inverse_bwt_result = inverse_BWT(inverse_mtf_result);
    inverse_bwt_result
}