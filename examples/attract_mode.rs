use rgba::Gba;

fn save_ppm(fb: &[u32], path: &str) {
    let mut out = format!("P6\n240 160\n255\n");
    let header = out.as_bytes().to_vec();
    let mut data = Vec::with_capacity(header.len() + 240 * 160 * 3);
    data.extend_from_slice(&header);
    for i in 0..240 * 160 {
        let p = fb[i];
        data.push((p & 0xFF) as u8);
        data.push(((p >> 8) & 0xFF) as u8);
        data.push(((p >> 16) & 0xFF) as u8);
    }
    std::fs::write(path, &data).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run without any input - let attract mode play
    for frame in 0..5000u32 {
        gba.run_frame_parallel(&mut fb);

        // Save screenshots at key frames
        if [
            200, 300, 400, 500, 600, 800, 1000, 1500, 2000, 3000, 4000, 4999,
        ]
        .contains(&frame)
        {
            let path = format!("/tmp/attract_f{}.ppm", frame);
            save_ppm(&fb, &path);

            let mut colors = std::collections::HashMap::new();
            for &p in &fb {
                *colors.entry(p).or_insert(0u32) += 1;
            }
            let n = colors.len();
            let top = colors.iter().max_by_key(|(_, &n)| n);
            if let Some((&c, &count)) = top {
                let pct = count as f64 / (240.0 * 160.0) * 100.0;
                println!(
                    "Frame {}: {} colors, top={:#010X} ({:.0}%)",
                    frame, n, c, pct
                );
            }
        }
    }
}
