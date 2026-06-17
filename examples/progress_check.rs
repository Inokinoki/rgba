use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);

        if frame % 60 == 0 || frame == 240 || frame == 192 || frame == 300 {
            let ppu = gba.ppu();
            let dispcnt = ppu.get_dispcnt();
            let mode = dispcnt & 7;

            let mut color_counts: std::collections::HashMap<u32, usize> =
                std::collections::HashMap::new();
            for &p in &fb {
                *color_counts.entry(p).or_insert(0) += 1;
            }
            let unique = color_counts.len();
            let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
            let green: usize = fb.iter().filter(|&&p| p == 0x0000FF00).count();

            println!(
                "Frame {:3}: DC={:04X} mode={} unique={} nonzero={} green={} ({:.0}%)",
                frame,
                dispcnt,
                mode,
                unique,
                nonzero,
                green,
                green as f64 / 384.0
            );
        }
    }

    // Save final frame
    let mut out = Vec::new();
    out.extend_from_slice(b"P6\n240 160\n255\n");
    for y in 0..160usize {
        for x in 0..240usize {
            let p = fb[y * 240 + x];
            out.extend_from_slice(&[((p >> 16) as u8), ((p >> 8) as u8), (p as u8)]);
        }
    }
    std::fs::write("/tmp/game_600frames.ppm", &out).unwrap();

    // Convert to PNG
    std::process::Command::new("python3")
        .args(["-c", "from PIL import Image; Image.open('/tmp/game_600frames.ppm').save('/tmp/game_600frames.png')"])
        .output()
        .ok();

    println!("Saved /tmp/game_600frames.png");
}
