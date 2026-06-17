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

    // Advance to round 74 (garbled state)
    for round in 0..75 {
        gba.input.press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
        gba.input.release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    }

    // Sync and dump screen entries
    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();
    let dispcnt = ppu.get_dispcnt();
    let bg_en = (dispcnt >> 8) & 0xF;

    for bg in 0..4 {
        if (bg_en >> bg) & 1 == 0 { continue; }
        let bgcnt = ppu.get_bgcnt(bg);
        let scr_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        
        println!("\nBG{}: scr_base={:#06X} hofs={} vofs={}", bg, scr_base, hofs, vofs);
        
        // Dump first 5 rows of screen entries (32 entries per row)
        for row in 0..5u16 {
            let mut row_entries = Vec::new();
            for col in 0..32u16 {
                let offset = scr_base + ((row as usize) * 32 + col as usize) * 2;
                let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
                let tile = entry & 0x3FF;
                row_entries.push(format!("{:4}", tile));
            }
            println!("  Row {}: {}", row, row_entries.join(" "));
        }
        
        // Count unique tiles
        let mut unique = std::collections::HashSet::new();
        for i in 0..1024 {
            let offset = scr_base + i * 2;
            if offset + 1 >= vram.len() { break; }
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            unique.insert(entry & 0x3FF);
        }
        println!("  Unique tiles: {}", unique.len());
        
        // Check which tiles have data
        let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let mut tiles_with_data = 0;
        for &t in &unique {
            let off = tile_base + t as usize * 32;
            if off + 32 <= vram.len() {
                let mut has = false;
                for b in 0..32 { if vram[off + b] != 0 { has = true; break; } }
                if has { tiles_with_data += 1; }
            }
        }
        println!("  Referenced tiles with data: {}/{}", tiles_with_data, unique.len());
    }
}
