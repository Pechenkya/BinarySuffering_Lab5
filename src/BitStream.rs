use std::cmp::min;
use std::io::{Read, Seek, Write};
use std::path::Path;
use std::fs::{File, OpenOptions};

// Max size of buffer in buffered read (4KB)
const BUFF_MAX_BYTE_SIZE: usize = 4096;

fn create_error(message: &str) -> Result<(), std::io::Error> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, message))
}

pub fn bin_string_LSBF(bytes: &[u8]) -> String {
    let result: Vec<String> = bytes
        .iter()
        .map(|byte| format!("{:08b}", byte.reverse_bits()))
        .collect();
    
    return result.join(" ");
}

pub struct BitStream {
    buff: Vec<u8>,
    bit_pointer: usize,
    read_dir: bool,
    file: File,
    byte_chunk_size: usize
}

impl BitStream {
    pub fn new(file_path: &str, read_dir: bool) -> Self {
        let buff: Vec<u8>;
        
        let file_stream: File;
        if read_dir {
            file_stream = File::open(file_path).unwrap();
            buff = vec![0u8; BUFF_MAX_BYTE_SIZE];
        }
        else {
            file_stream = OpenOptions::new().read(true)
                                            .create(true)
                                            .write(true)
                                            .truncate(false)
                                            .open(file_path).unwrap();
            buff = Vec::new();
        }

        BitStream {
            buff: buff,
            bit_pointer: 0,
            read_dir: read_dir,
            file: file_stream,
            byte_chunk_size: 0
        }
    }

    pub fn clear_output_file(&self) -> Result<(), std::io::Error> {
        if !self.read_dir {
            // Truncate file data
            self.file.set_len(0)?;
            Ok(())
        }
        else {
            create_error("Cannot clear file in read mode")
        }
    }

    pub fn write_bit_sequence(&mut self, in_buff: &[u8], bit_len: usize) -> Result<(), std::io::Error> {
        if self.read_dir {
            return create_error("This BitStream is in read mode");
        }

        let basic_shift = self.bit_pointer % 8;
        let full_bytes_to_write = bit_len / 8;
        let remaining_bits = bit_len % 8;
        
        if basic_shift == 0 {
            // Move full bytes to the stream buffer
            for i in 0..full_bytes_to_write {
                self.buff.push(in_buff[i]);
            }
            
            // Handle remaining bits
            if remaining_bits != 0 {
                self.buff.push((in_buff[full_bytes_to_write] << (8 - remaining_bits)) >> (8 - remaining_bits));
            }
        }
        else {
            let mut last_byte_id = self.buff.len() - 1;

            // Move full bytes to the stream buffer
            for i in 0..full_bytes_to_write {
                // Append low bits to the last byte
                self.buff[last_byte_id] |= in_buff[i] << basic_shift;

                // Push high bits as a new byte
                self.buff.push(in_buff[i] >> (8 - basic_shift));
                last_byte_id += 1;
            }

            // Handle remaining bits
            if remaining_bits != 0 {
                let last_input_byte_id = (bit_len + 7) / 8 - 1;
                if remaining_bits + basic_shift > 8 {
                    self.buff[last_byte_id] |= in_buff[last_input_byte_id] << basic_shift;
                    self.buff.push((in_buff[last_input_byte_id] << (8 - remaining_bits)) >> (16 - remaining_bits - basic_shift));
                }
                else {
                    self.buff[last_byte_id] |= (in_buff[last_input_byte_id] << (8 - remaining_bits)) >> (8 - remaining_bits - basic_shift);
                }
            }
        }
        self.bit_pointer += bit_len;

        // println!("Buffer after write (LSB-F): {}", bin_string_LSBF(&self.buff));
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        if self.read_dir {
            return create_error("BitStream cannot be flushed in read mode");
        }

        // println!("Buffer on flush (LSB-F): {}", bin_string_LSBF(&self.buff));

        self.file.write_all(&self.buff)?;
        self.file.flush()?;

        self.buff.clear();
        self.bit_pointer = 0;

        Ok(())
    }

    pub fn read_bit_sequence(&mut self, size: usize) -> Result<Vec<u8>, std::io::Error> {
        if !self.read_dir {
            return Err(create_error("This BitStream is in write mode").err().unwrap());
        }

        let mut result: Vec<u8> = Vec::new();
        let mut bits_read: usize = 0;

        while bits_read != size {
            // If chunk is empty -> read next
            if (self.byte_chunk_size * 8 - self.bit_pointer) == 0 {
                let bytes_read = self.file.read(&mut self.buff)?;
                if bytes_read == 0 {
                    // println!("Warning! Reached EOF for stream in read operation!");
                    return Ok(result);
                }
                self.byte_chunk_size = bytes_read;
                self.bit_pointer = 0;
            }

            let bit_chunk_size = self.byte_chunk_size * 8;
            let basic_shift = self.bit_pointer % 8;
            
            let bits_to_move = min(bit_chunk_size - self.bit_pointer, size - bits_read);
            let start_id = self.bit_pointer / 8;
            let end_id = (self.bit_pointer + bits_to_move + 7) / 8;

            // Copy all the bytes from start to end
            let curr_start_id = result.len();
            result.append(&mut self.buff[start_id..end_id].to_vec());

            // Set bytes in correct position
            if basic_shift != 0 {
                for i in curr_start_id..result.len() - 1 {
                    result[i] = (result[i] >> basic_shift) | (result[i + 1] << (8 - basic_shift));
                }

                let last_id = result.len() - 1;
                result[last_id] >>= basic_shift;
            }

            // Remove last byte if it is not used
            if (bits_to_move + 7) / 8 < (result.len() - curr_start_id) {
                result.pop();
            }

            // Clear unused high bits in the last byte
            let rem_bits = (bits_to_move % 8) as u8;
            if rem_bits != 0 {
                let last_id = result.len() - 1;
                result[last_id] = (result[last_id] << (8 - rem_bits)) >> (8 - rem_bits);
            }

            // If we have previous chunk in result, need to merge bytes
            if curr_start_id != 0 && basic_shift != 0 {
                let shift_in_chunks = (bits_read % 8) as u8;
                if shift_in_chunks != 0 {
                    for i in curr_start_id..result.len() {
                        result[i - 1] |= result[i] << (8 - shift_in_chunks);
                        result[i] = result[i] >> shift_in_chunks;
                    }

                    if rem_bits <= (8 - shift_in_chunks) {
                        result.pop();
                    }
                }
            }

            bits_read += bits_to_move;
            self.bit_pointer += bits_to_move;
        }

        Ok(result)
    }

    pub fn rewind_read_stream(&mut self) -> Result<(), std::io::Error> {
        if !self.read_dir {
            return create_error("Cannot reset stream in write mode");
        }

        self.file.rewind()?;
        self.buff.clear();
        self.buff.resize(BUFF_MAX_BYTE_SIZE, 0u8);
        
        self.bit_pointer = 0;
        self.byte_chunk_size = 0;

        Ok(())
    }
}