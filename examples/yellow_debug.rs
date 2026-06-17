use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Same sequence as full_capture to get dialogue_1
    for t in [300, 500, 800].iter() {
        let mut frame = 0u32;
        while frame < *t {
            gba.run_frame_parallel(&mut fb);
            frame += 1;
        }
    }
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press A (dialogue_0)
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    // dialogue_1 = press A again
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = gba.ppu();
    let dispcnt = ppu.get_dispcnt();
    println!("DISPCNT={:04X}", dispcnt);

    // Find unique colors in framebuffer
    let mut color_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for &p in fb.iter() {
        *color_counts.entry(p).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = color_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    println!("\nTop 10 framebuffer colors:");
    for (i, (color, count)) in sorted.iter().take(10).enumerate() {
        let r = (**color >> 16) & 0xFF;
        let g = (**color >> 8) & 0xFF;
        let b = **color & 0xFF;
        println!(
            "  {}: #{:08X} (r={} g={} b={}) count={}",
            i, **color, r, g, b, count
        );
    }

    // Check what get_pixel_tile_mode returns for yellow pixels
    // Find first non-black pixel
    for y in 0..160u16 {
        for x in 0..240u16 {
            let p = fb[y as usize * 240 + x as usize];
            if p != 0 && p != 0x00FFFFFF {
                let color_555 = gba.get_pixel_tile_mode(x, y);
                let r555 = color_555 & 0x1F;
                let g555 = (color_555 >> 5) & 0x1F;
                let b555 = (color_555 >> 10) & 0x1F;
                println!("\nFirst non-black/white pixel at ({},{}): #{:08X} -> 555={:04X} (r={} g={} b={})",
                    x, y, p, color_555, r555, g555, b555);

                // Check BG/sprite layers
                for bg in 0..4 {
                    if let Some(c) = gba.get_bg_pixel(ppu, 0, bg, x, y) {
                        println!("  BG{}: {:04X}", bg, c);
                    }
                }
                if let Some((c, pri)) = gba.get_sprite_pixel(ppu, x, y) {
                    println!("  Sprite: {:04X} pri={}", c, pri);
                }
                return;
            }
        }
    }
    println!("No non-black/white pixels found!");
}
