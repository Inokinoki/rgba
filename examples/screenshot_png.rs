use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }
    for _ in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    let unique_colors: std::collections::HashSet<u32> = fb.iter().copied().collect();
    eprintln!("Unique colors: {}", unique_colors.len());

    let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
    eprintln!("Non-zero pixels: {}/{}", nonzero, 240 * 160);

    let bg_color = fb[0];
    eprintln!("Pixel (0,0): {:08X}", bg_color);

    let mut color_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &p in &fb {
        *color_counts.entry(p).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = color_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!("Top 10 colors:");
    for (color, count) in sorted.iter().take(10) {
        eprintln!("  {:08X}: {} pixels", color, count);
    }

    let width = 240u32;
    let height = 160u32;
    let mut png_data: Vec<u8> = Vec::new();

    let mut encoder = png::Encoder::new(std::io::Cursor::new(&mut png_data), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    let mut raw_pixels = Vec::with_capacity((width * height * 4) as usize);
    for &pixel in &fb {
        let b = (pixel & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let r = ((pixel >> 16) & 0xFF) as u8;
        raw_pixels.extend_from_slice(&[r, g, b, 0xFF]);
    }
    writer.write_image_data(&raw_pixels).unwrap();
    writer.finish().unwrap();

    std::fs::write("/tmp/game_after_fix.png", &png_data).unwrap();
    eprintln!(
        "Saved PNG to /tmp/game_after_fix.png ({} bytes)",
        png_data.len()
    );
}
