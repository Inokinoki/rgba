use rgba::Gba;
use rgba::KeyState;

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

    let mut frame = 0u32;

    // Boot through title
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }

    // Press START
    gba.input.press_key(KeyState::START);
    for _ in 0..4 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }

    // Save at this point
    save_ppm(&fb, "/tmp/seq_start.ppm");
    println!("Frame {}: saved seq_start", frame);

    // Press A progressively and save screenshots at key points
    for i in 0..60 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
            frame += 1;
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
            frame += 1;
        }

        if [0, 5, 10, 15, 20, 25, 30, 40, 50, 59].contains(&i) {
            let path = format!("/tmp/seq_a{}.ppm", i);
            save_ppm(&fb, &path);

            let mut colors = std::collections::HashMap::new();
            for &p in &fb {
                *colors.entry(p).or_insert(0u32) += 1;
            }
            let n_colors = colors.len();
            let top = colors.iter().max_by_key(|(_, &n)| n).map(|(&c, &n)| (c, n));
            if let Some((c, n)) = top {
                let pct = n as f64 / (240.0 * 160.0) * 100.0;
                println!(
                    "Frame {} (A#{}/60): {} colors, top={:#010X} ({:.0}%)",
                    frame,
                    i + 1,
                    n_colors,
                    c,
                    pct
                );
            }
        }
    }

    // Extra 120 frames
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }
    save_ppm(&fb, "/tmp/seq_final.ppm");

    let mut colors = std::collections::HashMap::new();
    for &p in &fb {
        *colors.entry(p).or_insert(0u32) += 1;
    }
    println!("Frame {} (final): {} colors", frame, colors.len());
}
