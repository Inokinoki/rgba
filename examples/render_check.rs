use rgba::Gba;
use rgba::KeyState;

fn save_bmp(framebuffer: &[u32], path: &str) {
    let width = 240u32;
    let height = 160u32;
    let row_size = (width * 4 + 3) & !3;
    let file_size = 54 + row_size * height;
    let mut bmp = vec![0u8; file_size as usize];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[2..6].copy_from_slice(&file_size.to_le_bytes());
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&width.to_le_bytes());
    bmp[22..26].copy_from_slice(&height.to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
    for y in 0..height {
        for x in 0..width {
            let src_idx = ((height - 1 - y) * width + x) as usize;
            let dst_idx = (54 + y * row_size + x * 4) as usize;
            let pixel = framebuffer[src_idx];
            bmp[dst_idx] = (pixel & 0xFF) as u8;
            bmp[dst_idx + 1] = ((pixel >> 8) & 0xFF) as u8;
            bmp[dst_idx + 2] = ((pixel >> 16) & 0xFF) as u8;
            bmp[dst_idx + 3] = ((pixel >> 24) & 0xFF) as u8;
        }
    }
    std::fs::write(path, &bmp).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    // Run to A-30
    for i in 0..31 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut fb);
        }

        // Capture every frame from A-25 to A-30 to find the most colorful one
        if i >= 25 {
            let unique: std::collections::HashMap<u32, u32> =
                fb.iter()
                    .fold(std::collections::HashMap::new(), |mut m, &p| {
                        *m.entry(p).or_insert(0) += 1;
                        m
                    });
            let non_black = fb.iter().filter(|&&p| p != 0).count();
            println!("A-{}: {} colors, {} non-black", i, unique.len(), non_black);
            save_bmp(&fb, &format!("/tmp/fix_a{}.bmp", i));
        }
    }

    // Check register state
    let dispcnt = gba.mem_read_word(0x04000000) as u16;
    let win0h = gba.mem_read_word(0x04000040) as u16;
    let win0v = gba.mem_read_word(0x04000042) as u16;
    let win1h = gba.mem_read_word(0x04000044) as u16;
    let win1v = gba.mem_read_word(0x04000046) as u16;
    let winin = gba.mem_read_word(0x04000048) as u16;
    let winout = gba.mem_read_word(0x0400004A) as u16;

    println!("\n=== Registers at A-30 ===");
    println!(
        "DISPCNT: 0x{:04X} (BG0={}, BG1={}, BG2={}, BG3={}, OBJ={}, WIN0={}, WIN1={}, OBJ_WIN={})",
        dispcnt,
        (dispcnt >> 3) & 1,
        (dispcnt >> 4) & 1,
        (dispcnt >> 5) & 1,
        (dispcnt >> 6) & 1,
        (dispcnt >> 7) & 1,
        (dispcnt >> 8) & 1,
        (dispcnt >> 9) & 1,
        (dispcnt >> 10) & 1
    );
    println!(
        "WIN0: left={} right={} top={} bottom={}",
        win0h & 0xFF,
        (win0h >> 8) & 0xFF,
        win0v & 0xFF,
        (win0v >> 8) & 0xFF
    );
    println!(
        "WIN1: left={} right={} top={} bottom={}",
        win1h & 0xFF,
        (win1h >> 8) & 0xFF,
        win1v & 0xFF,
        (win1v >> 8) & 0xFF
    );
    println!(
        "WININ:  0x{:04X} (WIN0={}, WIN1={})",
        winin,
        winin & 0x1F,
        (winin >> 8) & 0x1F
    );
    println!(
        "WINOUT: 0x{:04X} (outside={}, OBJ_win={})",
        winout,
        winout & 0x1F,
        (winout >> 8) & 0x1F
    );

    let vis = gba.ppu().get_window_visibility(120, 80);
    println!("Window visibility at (120,80): 0x{:04X}", vis);

    // Also scan all frames to find one with highest color count
    println!("\n=== Scanning for most colorful frame ===");
    let mut gba2 = Gba::new();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb2 = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba2.run_frame_parallel(&mut fb2);
    }
    gba2.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba2.run_frame_parallel(&mut fb2);
    }
    gba2.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba2.run_frame_parallel(&mut fb2);
    }

    let mut best_colors = 0;
    let mut best_frame = 0;
    for frame in 0..200 {
        gba2.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba2.run_frame_parallel(&mut fb2);
        }
        gba2.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba2.run_frame_parallel(&mut fb2);
        }

        let unique: std::collections::HashMap<u32, u32> =
            fb2.iter()
                .fold(std::collections::HashMap::new(), |mut m, &p| {
                    *m.entry(p).or_insert(0) += 1;
                    m
                });
        if unique.len() > best_colors {
            best_colors = unique.len();
            best_frame = frame;
        }
        if frame == best_frame && best_colors > 5 {
            save_bmp(&fb2, "/tmp/fix_best.bmp");
        }
    }
    println!("Best frame: A-{} with {} colors", best_frame, best_colors);
}
