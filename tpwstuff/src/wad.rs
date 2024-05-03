use std::collections::HashMap;
use std::ffi::CString;
use crate::parse::{be_u16, le_u32, take};

struct WadEntry {
    compression: u8,
    data: Vec<u8>,
}

pub struct WadFile {
    wad: HashMap<String, WadEntry>
}

impl WadFile {
    pub fn files(&self) -> Vec<String> {
        self.wad.keys().cloned().collect()
    }
    pub fn new(p: &str) -> Self {
        let mut wad = HashMap::new();

        let w = std::fs::read(p).unwrap();
        let (i, _) = take(&w, 4);
        let (i, _v) = le_u32(i);
        let (i, _p) = take(i, 64);
        let (i, fc) = le_u32(i);
        let (i, off) = le_u32(i);
        let (i, fll) = le_u32(i);
        let (i, _p1) = le_u32(i);


        let mut i = i;
        for _ in 0..fc {
            let (j, _) = le_u32(i);
            let (j, fno) = le_u32(j);
            let (j, fnl) = le_u32(j);

            let filename_byte = &w[fno as usize..][..fnl as usize];
            let filename = CString::from_vec_with_nul(filename_byte.to_vec()).unwrap().to_string_lossy().to_string();

            let (j, dof) = le_u32(j);
            let (j, fl) = le_u32(j);
            let (j, c ) = le_u32(j);
            let (j, _ds) = le_u32(j);
            let (j, _p) = take(j, 12);

            // if !filename.contains("MD2") {
            println!("fn = {filename:?}, c={c}");

            let data = w[dof as usize..][..fl as usize].to_vec();

            wad.insert(filename, WadEntry {
                compression: c as u8,
                data,
            });
            // }


            i = j;
        }

        println!("version: {_v}, file cnt: {fc}, off={off}, fll={fll}");

        Self {
            wad
        }
    }

    pub fn get_data(&self, name: &str) -> Vec<u8> {
        let w = self.wad.get(name).unwrap();

        if w.compression == 0 {
            w.data.clone()
        } else {
            {
                fn decompress_data(data: &[u8], output_data: &mut Vec<u8>, offset: usize, opcode_length: usize, proceeding_data_length: usize, referenced_data_length: usize, referenced_data_offset: usize )
                {
                    for i in 0..proceeding_data_length  // Proceeding data comes from the source buffer (compressed data)
                    {
                        let pos = offset + opcode_length + i;
                        if pos >= data.len() {
                            break; // Prevent any overflowing
                        }

                        output_data.push( data[pos] );
                    }

                    let output_data_len = output_data.len();

                    for i in 0..referenced_data_length // Referenced data comes from the output buffer (decompressed data)
                    {
                        if referenced_data_offset > output_data_len {
                            break;
                        }
                        let pos = output_data_len - referenced_data_offset;

                        output_data.push( output_data[pos + i] );
                    }
                }


                let mut out = Vec::new();

                let (i, h) = be_u16(&w.data);
                if h != 0x10FB {
                    panic!();
                }
                let (_i, _) = take(i, 3);

                let mut pos = 5;
                loop {
                    let cmd = w.data[pos];
                    // println!("{cmd:b}");

                    pos += 1;

                    let offset = pos - 1;

                    if cmd & 0b1000_0000 == 0b0000_0000 {
                        let data = &w.data;

                        let proceeding_data_length = data[offset] & 0x03;
                        let referenced_data_length = ((data[offset] & 0x1C) >> 2) + 3;
                        let referenced_data_distance = (((data[offset] & 0x60) as usize) << 3) + data[offset + 1] as usize + 1;
                        let skip_ahead = proceeding_data_length;

                        // println!("pdl = {proceeding_data_length}, rdl = {referenced_data_length}, rdd={referenced_data_distance}");

                        decompress_data(data, &mut out, offset, 2, proceeding_data_length as usize, referenced_data_length as usize, referenced_data_distance as usize);
                        pos += (2 + skip_ahead - 1) as usize;
                        // println!("2bc");


                    } else {
                        if ((cmd & 0x1F) + 1) << 2 <= 0x70 && cmd & 0b1110_0000 == 0b1110_0000 {
                            let proceeding_data_length = ((cmd & 0x1F) + 1) << 2;
                            decompress_data(&w.data, &mut out, pos - 1, 1, proceeding_data_length as usize, 0, 0);
                            pos += (proceeding_data_length - 1 + 1) as usize;
                            // println!("obc");
                        } else {
                            if cmd & 0b1110_0000 == 0b1100_0000 {
                                let data = &w.data;
                                let proceeding_data_length = data[offset] & 0x03;
                                let referenced_data_length = ((data[offset] as usize & 0x0C) << 6) + data[offset + 3] as usize + 5;
                                let referenced_data_distance = ((data[offset] as usize & 0x10) << 12) + ((data[offset + 1] as usize) << 8) + data[offset + 2] as usize + 1;
                                let skip_ahead = proceeding_data_length;



                                decompress_data(data, &mut out, offset, 4, proceeding_data_length as usize, referenced_data_length as usize, referenced_data_distance as usize);
                                pos += (4 - 1 + skip_ahead) as usize;

                                // println!("4bc");
                            } else {
                                if cmd & 0b1100_0000 == 0b1000_0000 {
                                    let data = &w.data;
                                    let proceeding_data_length = (data[offset + 1] & 0xC0) >> 6;
                                    let referenced_data_length = (data[offset] & 0x3F) + 4;
                                    let referenced_data_distance = (((data[offset + 1] & 0x3F) as usize) << 8) + data[offset + 2] as usize + 1;
                                    let skip_ahead = proceeding_data_length;



                                    decompress_data(data, &mut out, offset, 3, proceeding_data_length as usize, referenced_data_length as usize, referenced_data_distance as usize);
                                    pos += (3 - 1 + skip_ahead) as usize;

                                    // println!("3bc");
                                } else {
                                    if ((cmd & 0x1F) + 1) << 2 > 0x70 && cmd & 0b1110_0000 == 0b1110_0000 {
                                        let data = &w.data;

                                        let proceeding_data_length = data[offset] & 0x03;
                                        // let skipAhead = proceeding_data_length;
                                        decompress_data(data, &mut out, offset, 3, proceeding_data_length as usize, 0 as usize, 0 as usize);
                                        // pos += (1 - 1 + skipAhead) as usize;
                                        break;
                                    } else {
                                        panic!("bad cmd: {cmd:b}");
                                    }
                                }
                            }
                        }
                    }
                }

                out
            }
        }
    }
}