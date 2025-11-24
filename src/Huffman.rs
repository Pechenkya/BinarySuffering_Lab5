use crate::BitStream::BitStream;
use crate::TransformationMethods::*;

struct Node {
    weight: u32,
    byte_value: Option<u8>,
    parent: Option<Box<Node>>,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

pub struct HuffmanEncoder {
    freq_t: [u32; 256],
    root: Option<Box<Node>>,
    input_stream: BitStream,
    output_stream: BitStream,
    codes: [([u8; 32], u8); 256], // (code, code_length)
}

pub struct HuffmanDecoder {
    freq_t: [u32; 256],
    root: Option<Box<Node>>,
    input_stream: BitStream,
    output_stream: BitStream,
    codes: [([u8; 32], u8); 256], // (code, code_length)
}

impl HuffmanEncoder {
    fn calc_frequences(&mut self) {
        while let Ok(byte_arr) = self.input_stream.read_bit_sequence(8) {
            if byte_arr.len() == 0 {
                break;
            }

            self.freq_t[byte_arr[0] as usize] += 1;
        }

        self.input_stream.rewind_read_stream().unwrap();
    }

    fn build_tree_and_get_codes(&mut self) {
        let mut queue: Vec<Box<Node>> = Vec::new();
        for (i, &freq) in self.freq_t.iter().enumerate() {
            if freq != 0 {
                queue.push(Box::new(Node {
                    weight: freq,
                    byte_value: Some(i as u8),
                    parent: None,
                    left: None,
                    right: None,
                }));
            }
        }

        // Build Huffman tree
        queue.sort_by_key(|node| node.weight);
        println!("Initial queue: {:?}", queue.iter().map(|n| (n.byte_value, n.weight)).collect::<Vec<_>>());
        while queue.len() > 1 {
            queue.sort_by_key(|node| node.weight);
            let left = queue.remove(0);
            let right = queue.remove(0);

            let parent = Box::new(Node {
                weight: left.weight + right.weight,
                byte_value: None,
                parent: None,
                left: Some(left),
                right: Some(right),
            });

            queue.push(parent);
        }

        self.root = Some(queue.remove(0));

        // Traverse tree to get codes
        let mut stack: Vec<(&Node, [u8; 32], u8)> = Vec::new();
        stack.push((self.root.as_ref().unwrap(), [0; 32], 0));

        while let Some((node, acc_code, code_length)) = stack.pop() {
            if let Some(byte_value) = node.byte_value {
                self.codes[byte_value as usize] = (acc_code, code_length);
            } else {
                if let Some(ref right) = node.right {
                    let mut r_acc_code = acc_code.clone();
                    r_acc_code[(code_length / 8) as usize] |= 1 << (code_length % 8);
                    stack.push((right, r_acc_code, code_length + 1));
                }

                if let Some(ref left) = node.left {
                    stack.push((left, acc_code, code_length + 1));
                }
            }
        }
    }

    pub fn encode(input: &str, output: &str) {
        let mut internal_encoder = HuffmanEncoder {
            freq_t: [0; 256],
            root: None,
            input_stream: BitStream::new(input, true),
            output_stream: BitStream::new(output, false),
            codes: [([0; 32], 0); 256],
        };

        internal_encoder.output_stream.clear_output_file().unwrap();

        internal_encoder.calc_frequences();
        internal_encoder.build_tree_and_get_codes();
        
        {
            // Debug
            let mut deb_print = internal_encoder.codes.iter_mut().enumerate().filter(|(_, (code, length))| *length > 0)
                                                             .map(|(idx, (code, length))| {
                                                                 let code = &code[0..((*length as usize + 7) / 8)];
                                                                 (idx, length, format!("{:?}", code.to_vec()))
                                                             }).collect::<Vec<_>>();

            deb_print.sort_by_key(|(_, &mut l, _)| l);

            println!("Codes ({}): {:?}", deb_print.len(), deb_print);
            println!("Code for i: {:b}", internal_encoder.codes[b'i' as usize].0[0]);
            println!("Code for L: {:b}", internal_encoder.codes[b'L' as usize].0[0]);
            println!("Code for Space: {:b}", internal_encoder.codes[b' ' as usize].0[0]);
        }

        // Write frequency table to output
        for (i, freq) in internal_encoder.freq_t.iter().enumerate() {
            internal_encoder.output_stream.write_bit_sequence(&u32::to_le_bytes(*freq), 32).unwrap();
        }
        
        // Encode all bytes
        while let Ok(byte_arr) = internal_encoder.input_stream.read_bit_sequence(8) {
            if byte_arr.len() == 0 {
                break;
            }

            let byte = byte_arr[0];
            let (code, code_length) = internal_encoder.codes[byte as usize];

            internal_encoder.output_stream.write_bit_sequence(&code, code_length as usize).unwrap();
        }

        internal_encoder.output_stream.flush().unwrap();
    }
}

impl HuffmanDecoder {
    fn build_tree_and_get_codes(&mut self) {
        let mut queue: Vec<Box<Node>> = Vec::new();
        for (i, &freq) in self.freq_t.iter().enumerate() {
            if freq != 0 {
                queue.push(Box::new(Node {
                    weight: freq,
                    byte_value: Some(i as u8),
                    parent: None,
                    left: None,
                    right: None,
                }));
            }
        }

        // Build Huffman tree
        while queue.len() > 1 {
            queue.sort_by_key(|node| node.weight);
            let left = queue.remove(0);
            let right = queue.remove(0);

            let parent = Box::new(Node {
                weight: left.weight + right.weight,
                byte_value: None,
                parent: None,
                left: Some(left),
                right: Some(right),
            });

            queue.push(parent);
        }

        self.root = Some(queue.remove(0));

        // Traverse tree to get codes
        let mut stack: Vec<(&Node, [u8; 32], u8)> = Vec::new();
        stack.push((self.root.as_ref().unwrap(), [0; 32], 0));

        while let Some((node, acc_code, code_length)) = stack.pop() {
            if let Some(byte_value) = node.byte_value {
                self.codes[byte_value as usize] = (acc_code, code_length);
            } else {
                if let Some(ref right) = node.right {
                    let mut r_acc_code = acc_code;
                    r_acc_code[(code_length / 8) as usize] |= 1 << (code_length % 8);
                    stack.push((right, r_acc_code, code_length + 1));
                }

                if let Some(ref left) = node.left {
                    stack.push((left, acc_code, code_length + 1));
                }
            }
        }
    }

    pub fn decode(input: &str, output: &str) {
        let mut internal_decoder = HuffmanDecoder {
            freq_t: [0; 256],
            root: None,
            input_stream: BitStream::new(input, true),
            output_stream: BitStream::new(output, false),
            codes: [([0; 32], 0); 256],
        };

        internal_decoder.output_stream.clear_output_file().unwrap();
        
        // Read frequency table from input
        let table_bytes = internal_decoder.input_stream.read_bit_sequence(8192).unwrap();
        for (i, freq) in internal_decoder.freq_t.iter_mut().enumerate() {
            *freq = u32::from_le_bytes(table_bytes[i*4..i*4+4].try_into().unwrap());
        }
        
        internal_decoder.build_tree_and_get_codes();
        
        {
            // Debug
            let deb_print = internal_decoder.codes.iter_mut().enumerate().filter(|(_, (code, length))| *length > 0)
                                                             .map(|(idx, (code, length))| {
                                                                 let code = &code[0..((*length as usize + 7) / 8)];
                                                                 (idx, length, format!("{:?}", code.to_vec()))
                                                             }).collect::<Vec<_>>();

            println!("Code for i: {:b}", internal_decoder.codes[b'i' as usize].0[0]);
            println!("Code for L: {:b}", internal_decoder.codes[b'L' as usize].0[0]);
            println!("Code for Space: {:b}", internal_decoder.codes[b' ' as usize].0[0]);
        }

        // Decode all bytes
        let mut symbols_left: u32 = internal_decoder.freq_t.iter().sum();
        let mut current_node = internal_decoder.root.as_ref().unwrap();
        while let Ok(byte_arr) = internal_decoder.input_stream.read_bit_sequence(1) {
            if byte_arr.len() == 0 || symbols_left == 0 {
                break;
            }

            let read_val = byte_arr[0]; 
            current_node = if read_val == 0 {
                current_node.left.as_ref().unwrap()
            } else {
                current_node.right.as_ref().unwrap()
            };

            if let Some(byte_value) = current_node.byte_value {
                internal_decoder.output_stream.write_bit_sequence(&[current_node.byte_value.unwrap()], 8).unwrap();
                current_node = internal_decoder.root.as_ref().unwrap();

                symbols_left -= 1;
            }
        }

        internal_decoder.output_stream.flush().unwrap();
    }
}