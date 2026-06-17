use rgba::Gba;

fn save_ppm(fb: &[u32], path: &str) {
    let width = 240;
    let height = 160;
    let mut out = Vec::with_capacity(width * height * 3 + 100);
    out.extend_from_slice(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    for y in 0..height {
        for x in 0..width {
            let pixel = fb[y * width + x];
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            out.extend_from_slice(&[r, g, b]);
        }
    }
    std::fs::write(path, &out).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: run_single <rom_path> [output.ppm]");
        return;
    }
    let rom_path = &args[1];
    let out_path = if args.len() > 3 {
        args[3].clone()
    } else {
        "/tmp/test_out.ppm".to_string()
    };

    let mut gba = Gba::new();
    if let Err(e) = gba.load_rom_path(rom_path) {
        eprintln!("Load error: {:?}", e);
        return;
    }

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }

    save_ppm(&fb, &out_path);
    eprintln!("Saved to {}", out_path);

    // Check for pass/fail by looking at pixel patterns
    // Most test ROMs show "Passed" or "Failed" text
    // Simple heuristic: count red vs green pixels in bottom half
    let mut red = 0u32;
    let mut green = 0u32;
    let mut white = 0u32;
    for y in 0..160 {
        for x in 0..240 {
            let p = fb[y * 240 + x];
            let r = (p >> 16) & 0xFF;
            let g = (p >> 8) & 0xFF;
            let b = p & 0xFF;
            if r > 200 && g < 80 && b < 80 {
                red += 1;
            }
            if g > 200 && r < 80 && b < 80 {
                green += 1;
            }
            if r > 200 && g > 200 && b > 200 {
                white += 1;
            }
        }
    }
    eprintln!("Red: {}, Green: {}, White: {}", red, green, white);
}
