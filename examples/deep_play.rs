use rgba::Gba;
use rgba::KeyState;

fn count_tiles(vram: &[u8]) -> u32 {
    let mut count = 0u32;
    for t in 0..1024u32 {
        let start = t as usize * 32;
        if start + 32 > vram.len() { break; }
        let mut has = false;
        for b in 0..32 { if vram[start + b] != 0 { has = true; break; } }
        if has { count += 1; }
    }
    count
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];
    let mut frame = 0u32;

    // Boot
    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

    // Press START
    gba.input.press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    gba.input.release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

    let tiles = count_tiles(gba.mem().vram());
    println!("After START: frame={} tiles={}", frame, tiles);

    // Spam A + direction to try to advance through ALL menus  
    for round in 0..200 {
        // Alternate between A presses and directional input
        if round % 10 < 7 {
            gba.input.press_key(KeyState::A);
            for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
            gba.input.release_key(KeyState::A);
        } else {
            // Try different directions
            let dir = match round % 4 {
                0 => KeyState::UP,
                1 => KeyState::DOWN,
                2 => KeyState::LEFT,
                _ => KeyState::RIGHT,
            };
            gba.input.press_key(dir);
            for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
            gba.input.release_key(dir);
        }
        
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

        if round % 20 == 19 {
            let tiles = count_tiles(gba.mem().vram());
            let green = 0x0000FF00u32;
            let mut non_green = 0u32;
            for &p in &framebuffer { if p != green && p != 0 { non_green += 1; } }
            println!("Round {}: frame={} tiles={} non_green={}", round, frame, tiles, non_green);
        }
    }

    // Save final screenshot
    let width = 240u32; let height = 160u32;
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
    std::fs::write("/tmp/game_deep_play.bmp", &bmp).unwrap();
    println!("Saved /tmp/game_deep_play.bmp at frame {}", frame);
}
