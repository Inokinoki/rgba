use rgba::Gba;
use rgba::KeyState;

fn save_bmp(fb: &[u32], path: &str) {
    let w = 240u32; let h = 160u32;
    let rs = (w * 4 + 3) & !3;
    let fs = 54 + rs * h;
    let mut bmp = vec![0u8; fs as usize];
    bmp[0..2].copy_from_slice(b"BM");
    bmp[2..6].copy_from_slice(&fs.to_le_bytes());
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&w.to_le_bytes());
    bmp[22..26].copy_from_slice(&h.to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
    for y in 0..h {
        for x in 0..w {
            let si = ((h - 1 - y) * w + x) as usize;
            let di = (54 + y * rs + x * 4) as usize;
            let p = fb[si];
            bmp[di] = (p & 0xFF) as u8;
            bmp[di+1] = ((p >> 8) & 0xFF) as u8;
            bmp[di+2] = ((p >> 16) & 0xFF) as u8;
            bmp[di+3] = ((p >> 24) & 0xFF) as u8;
        }
    }
    std::fs::write(path, &bmp).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];
    let mut frame = 0u32;

    // Boot
    for _ in 0..240 { gba.run_frame_parallel(&mut fb); frame += 1; }
    
    // START
    gba.input.press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut fb); frame += 1; }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 { gba.run_frame_parallel(&mut fb); frame += 1; }
    
    // Spam A for a long time to get through all menus
    for round in 0..500 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 { gba.run_frame_parallel(&mut fb); frame += 1; }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 { gba.run_frame_parallel(&mut fb); frame += 1; }
        
        // Save screenshots at specific intervals
        if [79, 150, 200, 250, 300, 350, 400, 450, 499].contains(&round) {
            let mut colors = std::collections::HashMap::new();
            for &p in &fb { *colors.entry(p).or_insert(0u32) += 1; }
            if colors.len() > 10 {
                save_bmp(&fb, &format!("/tmp/farm_r{}.bmp", round));
                println!("Round {}: {} colors", round, colors.len());
            }
        }
    }
    println!("Final frame: {}", frame);
}
