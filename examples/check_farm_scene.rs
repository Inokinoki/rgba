use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    // Run through more of the game to reach farm scene
    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Press START to skip intro
    gba.input.press_key(rgba::KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input.release_key(rgba::KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Press A through dialog choices
    for _ in 0..20 {
        gba.input.press_key(rgba::KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input.release_key(rgba::KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    // Wait longer for scene transition
    for _ in 0..500 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Save screenshot
    let mut data = Vec::new();
    data.extend_from_slice(b"P6\n240 160\n255\n");
    for i in 0..240 * 160 {
        let p = framebuffer[i];
        data.push((p & 0xFF) as u8);
        data.push(((p >> 8) & 0xFF) as u8);
        data.push(((p >> 16) & 0xFF) as u8);
    }
    std::fs::write("/tmp/farm_scene.ppm", &data).unwrap();

    gba.sync_ppu_full();
    let io = gba.mem().io();
    let dc = u16::from_le_bytes([io[0], io[1]]);
    println!(
        "DISPCNT={:#06X} mode={} BGs={:04b} OBJ={}",
        dc,
        dc & 7,
        (dc >> 8) & 0xF,
        (dc >> 12) & 1
    );

    for bg in 0..4 {
        let bgcnt_off = 0x08 + bg * 2;
        let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
        let screen_base = ((bgcnt >> 8) & 0x1F) as u32;
        let char_base = ((bgcnt >> 2) & 3) as u32;
        let size = bgcnt & 3;
        let pal_mode = (bgcnt >> 7) & 1;
        let priority = bgcnt & 3;
        println!(
            "BG{}: pri={} screen={:#010X} char={:#010X} size={} 256color={}",
            bg,
            priority,
            0x06000000 + screen_base * 0x400,
            0x06000000 + char_base * 0x4000,
            size,
            pal_mode
        );
    }

    // Count unique colors in framebuffer
    let mut colors = std::collections::HashMap::new();
    for &p in &framebuffer {
        *colors.entry(p).or_insert(0u32) += 1;
    }
    println!("\nUnique colors: {}", colors.len());
    let mut sorted: Vec<_> = colors.iter().collect();
    sorted.sort_by_key(|(_, &c)| std::cmp::Reverse(c));
    for (color, count) in sorted.iter().take(10) {
        println!("  {:08X}: {} pixels", color, count);
    }
}
