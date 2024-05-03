pub mod parse;
pub mod wad;
mod md2;

use clap::{Arg, Command};
use crate::parse::{le_f32, le_u16, le_u32, take};
use crate::wad::WadFile;

#[derive(Copy, Clone, Debug, Default)]
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

fn main() {
    let cmd = Command::new("tpw")
        .subcommand(Command::new("unwad").arg(Arg::new("file").required(true)))
                        .get_matches();

    if let Some(("unwad", m)) = cmd.subcommand() {
        let w = WadFile::new(m.get_one::<String>("file").unwrap());
        for file in w.files() {
            let d = w.get_data(&file);
            std::fs::write(format!("./out/{file}"), &d).unwrap();
        }
    }

    // let w = WadFile::new("/Users/cub3d/Downloads/mp/Theme Park World/data/levels/jungle/terrain.wad");
    // let d = w.get_data("grd_top1.wct");
    // std::fs::write("./not_tpw/grd_top1.wct", &d).unwrap();
    // let d = w.get_data("jri_lak1.wct");
    // std::fs::write("./not_tpw/jri_lak1.wct", &d).unwrap();
    // let d = w.get_data("gte_wal2.wct");
    // std::fs::write("./not_tpw/gte_wal2.wct", &d).unwrap();

    // let w = WadFile::new("./ui.wad");
    // let d = w.get_data("tpwlogo.MD2");
    // std::fs::write("./not_tpw/tpwlogo.MD2", &d).unwrap();
    // let d = std::fs::read("../tpwlogo.MD2").unwrap();
    // let d = std::fs::read("../b_map.MD2").unwrap();
    // let d = w.get_data("b_map.MD2");
    // std::fs::write("./not_tpw/b_map.MD2", &d).unwrap();

    // let w = WadFile::new("./volcano.wad");
    // let d = w.get_data("volcano.MD2");

    // let w = WadFile::new("monkey.wad");
    // let d = w.get_data("monkey.MD2");

    // let d = w.get_data("volcanoc.MD2");
    // std::fs::write("./not_tpw/volcanoc.MD2", &d).unwrap();


    // let w = WadFile::new("spider.wad");
    // let d = w.get_data("spider.MD2");
    //
    // let w = WadFile::new("./incagod.wad");
    // let d = w.get_data("incagod.MD2");

    let wad = WadFile::new("/mnt/Data/Programs/themeparkworld/data/levels/jungle/rides/tvsim.wad");
    let d = wad.get_data("Ptvsim.md2");

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
        let (_i, _off12) = le_u32(i);


        // Mesh
        let mesh_data = &d[mesh_ptr as usize..][..];

        println!("mesh_dat = {:x}, mesh_cnt={mesh_cnt}", mesh_ptr + 21 * 4);


        #[derive(Default, Clone, Debug)]
        #[allow(dead_code)]
        pub struct Mesh {
            name: String,
            posoff: u32,
            vert_cnt: u32,
            xoff: u32,
            yoff: u32,
            face_cnt: u32,
            idk_1: u32,
            vertex_order_len: u32,
            vertex_order_offset: u32,
            trans: Vec4,
            scale: Vec4,
        }
        let mut meshes = vec![Mesh::default(); mesh_cnt as usize];
        let mut i = mesh_data;


        fn take_vec4(i: &[u8]) -> (&[u8], Vec4) {
            let (i, x) = le_f32(i);
            let (i, y) = le_f32(i);
            let (i, z) = le_f32(i);
            let (i, w) = le_f32(i);
            (i, Vec4 {
                x,
                y,
                z,
                w
            })
        }

        for mesh_idx in 0..mesh_cnt {
            let (j, _idk) = le_u32(i);
            let (j, _self_ptr) = le_u32(j);
            let (j, _nxt_ptr) = le_u32(j);
            let (j, _idk_ptr) = le_u32(j);
            let (j, _pos) = take_vec4(j);
            let (j, _scale) = take_vec4(j);
            let (j, _rot) = take_vec4(j);
            let (j, _trans) = take_vec4(j);
            let (j, _) = le_u32(j);

            let (j, _noff) = le_u32(j); //+54
            let (j, idk_1) = le_u16(j);
            let (j, vert_cnt) = le_u16(j);
            let (j, y_cnt) = le_u16(j);
            let (j, z_cnt) = le_u16(j);       // some_count
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
            let (j, zoff) = le_u32(j);
            let (j, _38) = le_f32(j);
            let (j, _39) = le_f32(j);

            println!("Mesh {mesh_idx}");

            let name = d[_noff as usize..][..8].to_vec();
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
                face_cnt: y_cnt as _,
                idk_1: idk_1 as _,
                vertex_order_len: z_cnt as _,
                vertex_order_offset: zoff as _,
                trans: _trans,
                scale: _pos
            };

            i = j;
        }

        for (msh_idx, msh) in meshes.iter().enumerate() {
            let mut ooo = String::new();

            // let msh_idx = 9;
            // let msh = meshes.get(msh_idx).unwrap();
            println!("Exporting: {msh:?}");
            let msh_pos_end = meshes.get(msh_idx + 1).map(|m| m.posoff).unwrap_or(meshes.get(0).unwrap().xoff);
            let cnt = (msh_pos_end - msh.posoff) / (3 * 4 * 4);
            println!("msh pnt cnt = {cnt}");
            println!("idk1 = {}", msh.idk_1);

            let pos_data = &d[msh.posoff as usize..][..];

            let mut i = pos_data;

            let mut verticies = Vec::new();
            let mut c = msh.idk_1;
            if c % 4 == 0 {
                c = c;
            } else if c % 2 == 0 {
                c = c + 2;
            } else {
                c = c + 3;
            }


            loop {
                let elem = if c > 4 {
                    4
                } else {
                    c
                } as usize;
                let mut points = vec![(0_f32, 0_f32, 0_f32); elem];
                for idx in 0..elem {
                    let (j, f) = le_f32(i);
                    points[idx].0 = f;
                    i = j;
                }
                for idx in 0..elem {
                    let (j, f) = le_f32(i);
                    points[idx].1 = f;
                    i = j;
                }
                for idx in 0..elem {
                    let (j, f) = le_f32(i);
                    points[idx].2 = f;
                    i = j;
                }

                verticies.extend_from_slice(&points);

                if c < 4 {
                    break;
                }

                c -= 4;
                if c == 0 {
                    break;
                }
            }


            println!("z_cnt = {}", msh.vertex_order_len);
            println!("zoff = {}", msh.vertex_order_offset);

            // Verticies in vertex section are not stored in order, they need to be re-mapped
            // vertex at idx `x` in file data needs to be at position `vertex_order[x]`
            // Faces are specified in terms of this re-mapped ordering
            let mut vertex_order = vec![0u16; msh.vertex_order_len as _];
            let vertex_order_data = &d[msh.vertex_order_offset as usize..][..msh.vertex_order_len as usize * 2];
            let mut i = vertex_order_data;
            for idx in 0..msh.vertex_order_len as usize {
                let (j, zv) = le_u16(i);
                vertex_order[idx] = zv;
                i = j;
            }

            // Re-order verticies to line up with expected order
            let p = verticies.clone();
            let mut verticies = Vec::new();
            for x in vertex_order {
                verticies.push(p[x as usize]);
            }

            // Apply transforms
            {
                for (x, y, z) in &mut verticies {
                    *x += msh.trans.x;
                    *y += msh.trans.y;
                    *z += msh.trans.z;
                }
            }

            println!("pnt cnt = {}", verticies.len() * 3);

            // Parse face data
            let msh = meshes.get(msh_idx).unwrap();

            let face_data = &d[msh.yoff as usize..][..msh.face_cnt as usize * 8];

            let mut faces = Vec::new();

            let mut i = face_data;

            for _idx in 0..msh.face_cnt as usize {
                let (j, _ptr) = le_u16(i);
                let (j, a) = le_u16(j);
                let (j, b) = le_u16(j);
                let (j, c) = le_u16(j);

                faces.push((a + 1, b + 1, c + 1));

                i = j;
            }

            for (x, y, z) in &verticies {
                ooo.push_str(&format!("v {x} {y} {z}\n"));
            }

            for (x, y, z) in faces {
                ooo.push_str(&format!("f {x} {y} {z}\n"));
            }

            std::fs::write(format!("out/msh_{msh_idx}.obj"), &ooo).unwrap();
        }
    }

    panic!();


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

    /*

    // let d = w.get_data("base.map");
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
    }*/

 */
}
