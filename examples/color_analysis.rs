use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];
    let mut frame = 0u32;

    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

    gba.input.press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    gba.input.release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

    for round in 0..200 {
        if round % 10 < 7 {
            gba.input.press_key(KeyState::A);
            for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
            gba.input.release_key(KeyState::A);
        } else {
            let dir = match round % 4 {
                0 => KeyState::UP, 1 => KeyState::DOWN,
                2 => KeyState::LEFT, _ => KeyState::RIGHT,
            };
            gba.input.press_key(dir);
            for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
            gba.input.release_key(dir);
        }
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

        if round == 199 {
            let mut color_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
            for &p in &framebuffer {
                *color_counts.entry(p).or_insert(0) += 1;
            }
            let mut sorted: Vec<_> = color_counts.iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(a.1));
            
            println!("Frame {} - Top 10 colors:", frame);
            for (&color, &count) in sorted.iter().take(10) {
                let r = (color >> 16) & 0xFF;
                let g = (color >> 8) & 0xFF;
                let b = color & 0xFF;
                println!("  RGB({},{},{}) = {} pixels ({:.1}%)", r, g, b, count, count as f64 / 38400.0 * 100.0);
            }
            
            // Check a few specific pixels
            println!("\nSample pixels:");
            for y in [0u32, 40, 80, 120, 159] {
                for x in [0u32, 60, 120, 180, 239] {
                    let idx = (y * 240 + x) as usize;
                    let p = framebuffer[idx];
                    let r = (p >> 16) & 0xFF;
                    let g = (p >> 8) & 0xFF;
                    let b = p & 0xFF;
                    print!(" ({},{})={:02X}{:02X}{:02X}", x, y, r, g, b);
                }
                println!();
            }
        }
    }
}
