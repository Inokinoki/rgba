use rgba::Gba;

fn save_bmp(fb: &[u32], path: &str) {
    let w = 240u32;
    let h = 160u32;
    let rs = (w * 4 + 3) & !3;
    let fs = 54 + rs * h;
    let mut bmp = vec![0u8; fs as usize];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[2..6].copy_from_slice(&fs.to_le_bytes());
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&w.to_le_bytes());
    bmp[22..26].copy_from_slice(&h.to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
    for y in 0..h {
        for x in 0..w {
            let si = ((h - 1 - y) * w + x) as usize;
            let di = (54 + y * rs + x * 4) as usize;
            let p = fb[si];
            bmp[di] = (p & 0xFF) as u8;
            bmp[di + 1] = ((p >> 8) & 0xFF) as u8;
            bmp[di + 2] = ((p >> 16) & 0xFF) as u8;
            bmp[di + 3] = ((p >> 24) & 0xFF) as u8;
        }
    }
    std::fs::write(path, &bmp).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..195 {
        gba.run_frame_parallel(&mut fb);
    }
    save_bmp(&fb, "/tmp/frame195.bmp");

    for _ in 0..105 {
        gba.run_frame_parallel(&mut fb);
    }
    save_bmp(&fb, "/tmp/frame300.bmp");

    let mut colors = std::collections::HashMap::new();
    for &p in &fb {
        *colors.entry(p).or_insert(0u32) += 1;
    }
    println!("Frame 300: {} unique colors", colors.len());
    for (&c, &n) in colors.iter() {
        let pct = n as f64 / (240.0 * 160.0) * 100.0;
        if pct > 1.0 {
            println!("  color {:#010X}: {:.1}%", c, pct);
        }
    }
}
