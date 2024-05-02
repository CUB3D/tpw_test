pub mod parse;

use std::collections::HashMap;
use std::ffi::CString;
use crate::parse::{be_u16, le_f32, le_u16, le_u32, take};

struct WadEntry {
    compression: u8,
    data: Vec<u8>,
}

struct WadFile {
    wad: HashMap<String, WadEntry>
}

impl WadFile {
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
                fn DecompressData( data: &[u8], outputData: &mut Vec<u8>, offset: usize, opcodeLength: usize, proceedingDataLength: usize, referencedDataLength: usize, referencedDataOffset: usize )
                {
                    for i in 0..proceedingDataLength  // Proceeding data comes from the source buffer (compressed data)
                    {
                        let pos = (offset + opcodeLength + i);
                        if ( pos >= data.len() ) {
                            break; // Prevent any overflowing
                        }

                        outputData.push( data[pos] );
                    }

                    let outputDataLen = outputData.len();

                    for i in 0..referencedDataLength // Referenced data comes from the output buffer (decompressed data)
                    {
                        if (referencedDataOffset > outputDataLen) {
                            break;
                        }
                        let pos = (outputDataLen - referencedDataOffset);

                        outputData.push( outputData[pos + i] );
                    }
                }


                let mut out = Vec::new();

                let (i, h) = be_u16(&w.data);
                if(h != 0x10FB) {
                    panic!();
                }
                let (_i, _) = take(i, 3);

                let mut pos = 5;
                loop {
                    let cmd = w.data[pos];
                    // println!("{cmd:b}");

                    pos += 1;

                    let offset = pos - 1;

                    if(cmd & 0b1000_0000 == 0b0000_0000) {
                        let data = &w.data;

                        let proceedingDataLength = ((data[offset] & 0x03));
                        let referencedDataLength = (((data[offset] & 0x1C) >> 2) + 3);
                        let referencedDataDistance = ((((data[offset] & 0x60) as usize) << 3) + data[offset + 1] as usize + 1);
                        let skipAhead = proceedingDataLength;

                        // println!("pdl = {proceedingDataLength}, rdl = {referencedDataLength}, rdd={referencedDataDistance}");

                        DecompressData(data, &mut out, offset, 2, proceedingDataLength as usize, referencedDataLength as usize, referencedDataDistance as usize);
                        pos += (2 + skipAhead - 1) as usize;
                        // println!("2bc");


                    } else {
                        if ((cmd & 0x1F) + 1) << 2 <= 0x70 && cmd & 0b1110_0000 == 0b1110_0000 {
                            let proceedingDataLength = (((cmd & 0x1F) + 1) << 2);
                            DecompressData(&w.data, &mut out, pos - 1, 1, proceedingDataLength as usize, 0, 0);
                            pos += (proceedingDataLength - 1 + 1) as usize;
                            // println!("obc");
                        } else {
                            if cmd & 0b1110_0000 == 0b1100_0000 {
                                let data = &w.data;
                                let proceedingDataLength = ((data[offset] & 0x03));
                                let referencedDataLength = (((data[offset] as usize & 0x0C) << 6) + data[offset + 3] as usize + 5);
                                let referencedDataDistance = (((data[offset] as usize & 0x10) << 12) + ((data[offset + 1] as usize) << 8) + data[offset + 2] as usize + 1);
                                let skipAhead = proceedingDataLength;



                                DecompressData(data, &mut out, offset, 4, proceedingDataLength as usize, referencedDataLength as usize, referencedDataDistance as usize);
                                pos += (4 - 1 + skipAhead) as usize;

                                // println!("4bc");
                            } else {
                                if cmd & 0b1100_0000 == 0b1000_0000 {
                                    let data = &w.data;
                                    let proceedingDataLength = ((data[offset + 1] & 0xC0) >> 6);
                                    let referencedDataLength = ((data[offset] & 0x3F) + 4);
                                    let referencedDataDistance = ((((data[offset + 1] & 0x3F) as usize) << 8) + data[offset + 2] as usize + 1);
                                    let skipAhead = proceedingDataLength;



                                    DecompressData(data, &mut out, offset, 3, proceedingDataLength as usize, referencedDataLength as usize, referencedDataDistance as usize);
                                    pos += (3 - 1 + skipAhead) as usize;

                                    // println!("3bc");
                                } else {
                                    if ((cmd & 0x1F) + 1) << 2 > 0x70 && cmd & 0b1110_0000 == 0b1110_0000 {
                                        let data = &w.data;

                                        let proceedingDataLength = ((data[offset] & 0x03));
                                        // let skipAhead = proceedingDataLength;
                                        DecompressData(data, &mut out, offset, 3, proceedingDataLength as usize, 0 as usize, 0 as usize);
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

fn main() {
    // let w = WadFile::new("/Users/cub3d/Downloads/mp/Theme Park World/data/levels/jungle/terrain.wad");
    // let d = w.get_data("grd_top1.wct");
    // std::fs::write("./not_tpw/grd_top1.wct", &d).unwrap();
    // let d = w.get_data("jri_lak1.wct");
    // std::fs::write("./not_tpw/jri_lak1.wct", &d).unwrap();
    // let d = w.get_data("gte_wal2.wct");
    // std::fs::write("./not_tpw/gte_wal2.wct", &d).unwrap();

    // let w = WadFile::new("/Users/cub3d/Downloads/mp/Theme Park World/data/ui.wad");
    // let d = w.get_data("tpwlogo.MD2");
    // std::fs::write("./not_tpw/tpwlogo.MD2", &d).unwrap();
    // let d = w.get_data("b_map.MD2");
    // std::fs::write("./not_tpw/b_map.MD2", &d).unwrap();

    let w = WadFile::new("./volcano.wad");
    let d = w.get_data("volcano.MD2");

    // let w = WadFile::new("/Users/cub3d/Downloads/mp/Theme Park World/data/levels/jungle/rides/monkey.wad");
    // let d = w.get_data("monkey.MD2");

    // let d = w.get_data("volcanoc.MD2");
    // std::fs::write("./not_tpw/volcanoc.MD2", &d).unwrap();


    // let w = WadFile::new("spider.wad");
    // let d = w.get_data("spider.MD2");
    //
    // let w = WadFile::new("./incagod.wad");
    // let d = w.get_data("incagod.MD2");


    /*
struct hd {
u32 mg;
u32 v;
u32 fix;
u16 unk1;
u16 unk11;
u16 unk2;
u16 unk3;
u16 unk4;
u16 unk5;
char name[12];
u32 unk6;
u32 unk7;
u32 unk8;
u32 flag;
u16 cnt0;
u16 frame_cnt;
u16 cnt2_38;
u16 cnt3_3a;
u16 cnt4_3c;
u16 cnt5;
u16 cnt6;
u16 cnt7;
u16 mesh_cnt;
u16 cnt9;
u16 cnt10;
u16 off0;
u32 off1;
u32 off2;
u32 off3_54;
u32 off4_58;
u32 off5;
u32 off6;
u32 off7;
u32 off8;
u32 off9;
u32 mesh_ptr;
u32 off11;
u32 off12;
};

hd hd1 @ 0x0;

struct v3 {
float x[3];
};

struct frame_data {
u32 value;
u32 mustbezero;
u16 pad;
u16 willbe1;
u32 framenameoff;
};


struct texture_coord {
    u32 a;
    u32 b;
};
struct skin {
char name[20];
};

struct idk {
    u16 b[20];
    };




texture_coord texture_coords[hd1.frame_cnt] @ hd1.off2;
skin skin_names[hd1.frame_cnt] @  hd1.off2 + 8*hd1.frame_cnt;



frame_data frame_data[hd1.frame_cnt] @ hd1.off3_54;

// this is the overall mesh data, but you probs want the frames
//v3 tmp[hd1.cnt2_38*4] @ offs1.off4_58; // prob right sz and pos

if (hd1.off5 > 0) {
u8 tmp2[hd1.cnt3_3a*72] @ hd1.off5; // prob right pos
 }

 if (hd1.off7 > 0) {
idk tmp3[hd1.cnt4_3c] @ hd1.off7;
}

//u8 tmp4[hd1.cnt5*16] @ offs1.off6;

//u8 tmp5[hd1.cnt6*8] @ offs1.off9;

struct arr2 {
u32 a;
u32 b;
u32 c;
u32 _idk[21];
u32 p1;
u32 p2;
u32 p3;
u32 p_anotherstruct;
u32 p4;
u32 rest[11];
};

struct mesh {
u32 pad[21];
u32 nameoff;
u16 idk_1;
u16 idk_2;
u16 y_cnt;
u16 idk_3;
u32 posoff;
u32 _idk1;
u32 uvoff;
u32 xoff;
u32 yoff;
u32 idk2;
float _30;
float _31;
float _32;
float _33;
float _34;
float _35;
u32 _36;
u32 _37;
u32 _38;
u32 _39;
};

struct tri {
float x[4];
float y[4];
float z[4];
};

mesh meshes[hd1.mesh_cnt] @ hd1.mesh_ptr;

u32 cnt = (meshes[0].yoff - meshes[0].xoff)/4;
tri tris[cnt] @ meshes[0].posoff;
char meshname[8] @ meshes[2].nameoff;

struct y_ent {
u16 ptr;
u16 vals[3];
};

y_ent y[meshes[0].y_cnt] @  meshes[0].yoff;
     */

    let mut ooo = String::new();
    {
        let (i, _mag) = le_u32(&d);
        let (i, _v) = le_u32(i);
        let (i, _fix) = le_u32(i);
        let (i, _unk1) = le_u16(i);
        let (i, _unk11) = le_u16(i);
        let (i, _unk2) = le_u16(i);
        let (i, _unk3) = le_u16(i);
        let (i, _unk4) = le_u16(i);
        let (i, _unk5) = le_u16(i);
        let (i, _name) = take(i, 12);
        let (i, _unk6) = le_u32(i);
        let (i, _unk7) = le_u32(i);
        let (i, _unk8) = le_u32(i);
        let (i, _flg) = le_u32(i);
        let (i, _cnt0) = le_u16(i);
        let (i, _fr_cnt) = le_u16(i);
        let (i, _cnt2) = le_u16(i);
        let (i, _cnt3) = le_u16(i);
        let (i, _cnt4) = le_u16(i);
        let (i, _cnt5) = le_u16(i);
        let (i, _cnt6) = le_u16(i);
        let (i, _cnt7) = le_u16(i);
        let (i, mesh_cnt) = le_u16(i);
        let (i, _cnt9) = le_u16(i);
        let (i, _cnt10) = le_u16(i);
        let (i, _off0) = le_u16(i);
        let (i, _off1) = le_u32(i);

        let (i, _off2) = le_u32(i);
        let (i, _off3) = le_u32(i);
        let (i, _off4) = le_u32(i);
        let (i, _off5) = le_u32(i);
        let (i, _off6) = le_u32(i);
        let (i, _off7) = le_u32(i);
        let (i, _off8) = le_u32(i);
        let (i, _off9) = le_u32(i);
        let (i, mesh_ptr) = le_u32(i);
        let (i, _off11) = le_u32(i);
        let (i, _off12) = le_u32(i);


        // Mesh
        let mesh_data = &d[mesh_ptr as usize..][..];

        println!("mesh_dat = {:x}, mesh_cnt={mesh_cnt}", mesh_ptr + 21*4);


        #[derive(Default, Clone)]
        struct Mesh {
            name: String,
            posoff: u32,
            vert_cnt: u32,
            xoff: u32,
            yoff: u32,
            y_cnt: u32,
        }
        let mut meshes = vec![Mesh::default(); mesh_cnt as usize];
        let mut i = mesh_data;
        for mesh_idx in 0..mesh_cnt {
            let (j, _) = take(i, 21 * 4); // +0
            let (j, _noff) = le_u32(j); //+54
            let (j, _) = le_u16(j);
            let (j, vert_cnt) = le_u16(j);
            let (j, y_cnt) = le_u16(j);
            let (j, _) = le_u16(j);       // some_count
            let (j, _posoff) = le_u32(j); //5c
            let (j, _idk) = le_u32(j); // 60
            let (j, _uvoff) = le_u32(j); //64
            let (j, _xoff) = le_u32(j); //68
            let (j, _yoff) = le_u32(j); //6c
            let (j, _idk2) = le_u32(j);
            let (j, _30) = le_f32(j);
            let (j, _31) = le_f32(j);
            let (j, _32) = le_f32(j);
            let (j, _33) = le_f32(j);
            let (j, _34) = le_f32(j);
            let (j, _35) = le_f32(j);
            let (j, _36) = le_f32(j);
            let (j, _37) = le_f32(j);
            let (j, _38) = le_f32(j);
            let (j, _39) = le_f32(j);

            println!("Mesh {mesh_idx}");

            let mut name = d[_noff as usize..][..8].to_vec();
            let mut name_s = String::new();
            for b in name {
                if b == 0 {
                    break;
                }
                name_s.push(b as char);
            }

            println!("name = {name_s}");
            println!("y_cnt = {y_cnt}");
            println!("posoff = {_posoff}");
            println!("yoff = {_yoff}");

            meshes[mesh_idx as usize] = Mesh {
                vert_cnt: vert_cnt as u32,
                name: name_s,
                posoff: _posoff,
                xoff: _xoff,
                yoff: _yoff,
                y_cnt: y_cnt as _,
            };

            i = j;
        }

        let msh_idx = 0;
        let msh = meshes.get(msh_idx).unwrap();
        let msh_pos_end = meshes.get(msh_idx+1).map(|m| m.posoff).unwrap_or(meshes.get(0).unwrap().xoff);
        let cnt = (msh_pos_end - msh.posoff) / (3*4*4);
        println!("msh pnt cnt = {cnt}");

        let pos_data = &d[msh.posoff as usize..][..];

        let mut i = pos_data;

        let mut tris = Vec::new();
        for _ in 0..cnt {
            let mut points = vec![(0_f32, 0_f32, 0_f32); 4];
            for idx in 0..4 {
                let (j, f) = le_f32(i);
                points[idx].0 = f;
                i = j;
            }
            for idx in 0..4 {
                let (j, f) = le_f32(i);
                points[idx].1 = f;
                i = j;
            }
            for idx in 0..4 {
                let (j, f) = le_f32(i);
                points[idx].2 = f;
                i = j;
            }

            tris.push(points);
        }

        let msh = meshes.get_mut(msh_idx).unwrap();

        let face_data = &d[msh.yoff as usize..][..msh.y_cnt as usize *0x10];

        let mut faces = Vec::new();

        let mut i = face_data;
        // for x in 0..12 {
        //     let (j, x) = le_u16(i);
        //     faces.push(x);
        //     i = j;
        // }
        for idx in 0..msh.y_cnt as usize {
            let (j, _ptr) = le_u16(i);
            let (j, b) = le_u16(j);
            let (j, c) = le_u16(j);
            let (j, d) = le_u16(j);
            // let (j, e) = le_u16(j);
            // let (j, f) = le_u16(j);
            // let (j, g) = le_u16(j);


            // faces.push((b+1, c+1, d+1));
            faces.push((b+1, c+1, d+1));
            // if idx > 5 {
            //     break;
            // }
            // faces.push((e, f, g));


            i = j;
        }
        faces.clear();

        // let face2 = faces;
        // let mut faces = Vec::new();
        // faces.push((face2[1]+1, face2[2]+1, face2[3]+1));


        //f 6 1 5
        // f 6 5 10
        // 5 0 4
        // 5 4 9

        /*let mut i = face_data;
        for idx in 0..msh.y_cnt as usize {

            let (j, _ptr) = le_u16(i);
            let (j, b) = le_u16(j);
            let (j, c) = le_u16(j);
            let (j, d) = le_u16(j);
            // let (j, e) = le_u16(j);
            // let (j, f) = le_u16(j);
            // let (j, g) = le_u16(j);

            faces[idx].0 = b;
            faces[idx].1 = c;
            faces[idx].2 = d;


            i = j;
        }*/

        /*

        let _yoff = 0;
        let _xoff = 0;
        let _posoff = 0;
        let y_cnt = 0;

        println!("pos = {_posoff}");

        // mesh data
        let cnt = ((_yoff - _xoff)/4) as usize;
        println!("cnt = {cnt}");
        let pos_data = &d[_posoff as usize..][..];

        let mut i = pos_data;

        println!("xoff = {_xoff}, yoff={_yoff} posoff={_posoff}");
        let _posoff = _posoff + 24;
        let cnt2 = (_xoff - _posoff);
        println!("cnt2 = {cnt2}");

        let elems = 4_usize;

        let cnt_idk = (cnt2 as usize / (4*elems*3));

        println!("idk = {}", (cnt2 as f32 / (4*elems*3) as f32));
        println!("idk = {}", cnt_idk);

        for _ in 0..cnt_idk {
            let mut points = vec![(0_f32, 0_f32, 0_f32); elems];
            for idx in 0..elems {
                let (j, f) = le_f32(i);
                points[idx].0 = f;
                i = j;
            }
            for idx in 0..elems {
                let (j, f) = le_f32(i);
                points[idx].1 = f;
                i = j;
            }
            for idx in 0..elems {
                let (j, f) = le_f32(i);
                points[idx].2 = f;
                i = j;
            }

            tris.push(points);
        }*/
        /*
        let face_data = &d[_yoff as usize..][..y_cnt as usize *0x10];


        let mut faces = vec![(0u16, 0u16, 0u16); y_cnt as usize];

        println!("fce cnt = {}", y_cnt);
        let mut i = face_data;
        for idx in 0..y_cnt as usize {

            let (j, ptr) = le_u32(i);
            let (j, b) = le_u16(j);
            let (j, c) = le_u16(j);
            let (j, d) = le_u16(j);
            let (j, e) = le_u16(j);
            let (j, f) = le_u16(j);
            let (j, g) = le_u16(j);

            faces[idx].0 = b;
            faces[idx].1 = c;
            faces[idx].2 = d;


            i = j;
        }*/


        for tri in &tris {
            for (x, y, z) in tri {
                ooo.push_str(&format!("v {x} {y} {z}\n"));
            }
        }

        for (x,y,z) in faces {
            ooo.push_str(&format!("f {x} {y} {z}\n"));
        }


    }
    std::fs::write("./test.obj", &ooo).unwrap();

    panic!();

    /*
    struct hd {
u32 mg;
u32 v;
u32 fix;
u16 unk1;
u16 unk11;
u16 unk2;
u16 unk3;
u16 unk4;
u16 unk5;
char name[12];
u32 unk6;
u32 unk7;
u32 unk8;
u32 flag;
u16 cnt0;
u16 frame_cnt;
u16 cnt2_38;
u16 cnt3_3a;
u16 cnt4_3c;
u16 cnt5;
u16 cnt6;
u16 cnt7;
u16 cnt8_44;
u16 cnt9;
u16 cnt10;
u16 off0;
u32 off1;
u32 off2;
u32 off3_54;
u32 off4_58;
u32 off5;
u32 off6;
u32 off7;
u32 off8;
u32 off9;
u32 mesh_ptr;
u32 off11;
u32 off12;
};

hd hd1 @ 0x0;

struct v3 {
float x[3];
};

struct frame_data {
u32 value;
u32 mustbezero;
u16 pad;
u16 willbe1;
u32 framenameoff;
};


struct texture_coord {
    u32 a;
    u32 b;
};
struct skin {
char name[20];
};

struct idk {
    u16 b[20];
    };




texture_coord texture_coords[hd1.frame_cnt] @ hd1.off2;
skin skin_names[hd1.frame_cnt] @  hd1.off2 + 8*hd1.frame_cnt;



frame_data frame_data[hd1.frame_cnt] @ hd1.off3_54;

// this is the overall mesh data, but you probs want the frames
//v3 tmp[hd1.cnt2_38*4] @ offs1.off4_58; // prob right sz and pos

if (hd1.off5 > 0) {
u8 tmp2[hd1.cnt3_3a*72] @ hd1.off5; // prob right pos
 }

 if (hd1.off7 > 0) {
idk tmp3[hd1.cnt4_3c] @ hd1.off7;
}

//u8 tmp4[hd1.cnt5*16] @ offs1.off6;

//u8 tmp5[hd1.cnt6*8] @ offs1.off9;

struct arr2 {
u32 a;
u32 b;
u32 c;
u32 _idk[21];
u32 p1;
u32 p2;
u32 p3;
u32 p_anotherstruct;
u32 p4;
u32 rest[11];
};

struct mesh {
u32 pad[21];
u32 nameoff;
u16 idk[4];
u32 posoff;
u32 _idk1;
u32 uvoff;
u32 xoff;
u32 yoff;
u32 idk2;
float _30;
float _31;
float _32;
float _33;
float _34;
float _35;
};

struct tri {
float x[4];
float y[4];
float z[4];
};

mesh meshes[1] @ hd1.mesh_ptr;

u32 cnt = (meshes[0].yoff - meshes[0].xoff)/4;
tri tris[cnt] @ meshes[0].posoff;
char meshname[8] @ meshes[0].nameoff;

u32 cnt3 = (meshes[0].uvoff - meshes[0].yoff)/8;
u16 y[cnt3*4] @  meshes[0].yoff;
     */
    // fn img_hack(p: &str) -> anyhow::Result<Texture2D> {
    //     Horrid hack inbound
        // let f = std::fs::read(p).unwrap();
        // let (i, w) = le_u16(&f);
        // let (i, h) = le_u16(i);
        //
        // let mut i = i;
        //
        // let mut out = vec![(0.0, 0., 0, 0.); (w*h) as usize];
        // let mut imgbuf = image::ImageBuffer::new(w as u32, h as u32);
        //
        // let mut out = vec![0u8; (w as usize*h as usize*4)];
        // for xx in 0..w {
        //     for yy in 0..h {
        //         let (j, r) = le_f64(i);
        //         let (j, g) = le_f64(j);
        //         let (j, b) = le_f64(j);
        //         let (j, a) = le_f64(j);
        //         i = j;
        //
        //         out[(yy * w + xx) as usize * 4] = r as u8;
        //         out[(yy * w + xx) as usize * 4 + 1] = g as u8;
        //         out[(yy * w + xx) as usize * 4 + 2] = b as u8;
        //         out[(yy * w + xx) as usize * 4 + 3] = a as u8;
        //         imgbuf.put_pixel(xx as u32, yy as u32, image::Rgba([r as u8, g as u8, b as u8, a as u8]));
        //     }
        // }
        // imgbuf.save(format!("{}.png", p)).unwrap();
        // Texture2D::new_rgba(gl,  out, w as u32, h as _)
    // }
    //
    // let wct = img_hack(gl, "./not_tpw/jri_lak1.wct.raw")?;
    // let grd_top1 = img_hack(gl, "./not_tpw/grd_top1.wct.raw")?;
    // let grd_top1 = img_hack(gl, "./not_tpw/gte_wal1.wct.raw")?;


    let d = w.get_data("base.map");
    // std::fs::write("./not_tpw/out.txt", &d);
    let (i, _) = take(&d, 0x24);
    let (i, mag) = take(i, 4);
    assert_eq!(mag, b"MAP ");
    let (i,file_sz) = le_u32(i);
    assert!(i.len() >= file_sz as usize);
    let (i, width) = le_u32(i);
    let (i, height) = le_u32(i);
    let (i, _) = le_u32(i);
    let (i, _) = le_u32(i);
    let (i, _) = le_u32(i);
    let (i, _) = le_u32(i);
    let (i, _) = le_u32(i);

    let (i, map) = take(i, (width * height) as usize);
/*
    {
        // Create a new ImgBuf with width: imgx and height: imgy
        let mut imgbuf = image::ImageBuffer::new(128, 128);



        // A redundant loop to demonstrate reading image data
        for x in 0..128 {
            for y in 0..128 {
                let q = map[y*128+x];

                let ecs = uk.stage_manager_mut().current_mut().ecs_mut();
                let tile = ecs.spawn();
                ecs.attach(tile, Position::new((x * 16 )as _, (y * 16) as _)).unwrap();
                ecs.attach(tile, QuadRenderer::new(16., 16.)).unwrap();

                let p = match q {
                    0   => {
                        ecs.attach(tile, unknown4::ecs::components::texture_2d::Texture2D::new_handle(gl, &grd_top1).unwrap()).unwrap();

                        image::Rgb([0u8, 0, 0])
                    },
                    1   => {
                        ecs.attach(tile, SolidColourTexture::new(Colour::new_from_u32(0xFFFFFFFF))).unwrap();

                        image::Rgb([0x00, 0xFF, 0x00])
                    },
                    3   => {
                        ecs.attach(tile, unknown4::ecs::components::texture_2d::Texture2D::new_handle(gl, &wct).unwrap()).unwrap();
                        image::Rgb([0x00, 0x00, 0xFF])
                    },
                    8   => {
                        ecs.attach(tile, SolidColourTexture::new(Colour::new_from_u32(0xFFFFFFFF))).unwrap();

                        image::Rgb([0x00, 0x80, 0x80])
                    },
                    17  => {
                        ecs.attach(tile, SolidColourTexture::new(Colour::new_from_u32(0xFFFFFFFF))).unwrap();
                        image::Rgb([0x80, 0x80, 0x00])
                    },
                    16  => {
                        ecs.attach(tile, SolidColourTexture::new(Colour::new_from_u32(0xFFFFFFFF))).unwrap();
                        image::Rgb([0x80, 0x80, 0x80])
                    },
                    128  => {
                        ecs.attach(tile, SolidColourTexture::new(Colour::new_from_u32(0xFFFFFFFF))).unwrap();
                        image::Rgb([0xFF, 0x00, 0xFF])
                    },
                    144 => {
                        ecs.attach(tile, SolidColourTexture::new(Colour::new_from_u32(0xFFFFFFFF))).unwrap();
                        image::Rgb([0xFF, 0x80, 0x80])
                    },
                    148 => {
                        ecs.attach(tile, SolidColourTexture::new(Colour::new_from_u32(0xFFFFFFFF))).unwrap();
                        image::Rgb([0x80, 0x80, 0xFF])
                    },
                    _ =>   {
                        tracing::warn!("unk {q}");
                        image::Rgb([0xFF, 0xFf, 0xFF])
                    }
                };

                imgbuf.put_pixel(x as u32, y as u32, p);


            }
        }

        // Save the image as “fractal.png”, the format is deduced from the path
        imgbuf.save("./not_tpw/map.png").unwrap();
    }

 */
}
