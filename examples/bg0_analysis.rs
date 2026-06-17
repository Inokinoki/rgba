use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    // Run 300 frames
    for _ in 0..300 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    // Find what BG0 is configured to use
    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    let bg0cnt = u16::from_le_bytes([io[8], io[9]]);
    let bg1cnt = u16::from_le_bytes([io[0xA], io[0xB]]);
    let bg2cnt = u16::from_le_bytes([io[0xC], io[0xD]]);
    let bg3cnt = u16::from_le_bytes([io[0xE], io[0xF]]);

    println!("DISPCNT: {:04X}", dispcnt);
    for (i, cnt) in [bg0cnt, bg1cnt, bg2cnt, bg3cnt].iter().enumerate() {
        let priority = cnt & 3;
        let char_base = ((cnt >> 2) & 3) * 0x4000;
        let mosaic = (cnt >> 6) & 1;
        let bpp = if (cnt >> 7) & 1 != 0 { 8 } else { 4 };
        let screen_base = ((cnt >> 8) & 0x1F) * 0x800;
        let wrap = (cnt >> 13) & 1;
        let size = (cnt >> 14) & 3;
        println!("BG{}CNT: {:04X} (pri={} char_base=0x{:05X} mosaic={} {}bpp screen_base=0x{:05X} wrap={} size={})",
                 i, cnt, priority, char_base, mosaic, bpp, screen_base, wrap, size);
    }

    // Dump screen block entries for BG0
    let bg0_screen_base = ((bg0cnt >> 8) & 0x1F) as usize * 0x800;
    let vram = gba.mem().vram();
    println!(
        "\n=== BG0 screen block at 0x{:05X} (first 64 entries) ===",
        bg0_screen_base
    );
    for row in 0..8 {
        for col in 0..8 {
            let idx = row * 32 + col;
            let offset = bg0_screen_base + idx * 2;
            if offset + 2 <= vram.len() {
                let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
                let tile_num = entry & 0x3FF;
                let hflip = (entry >> 10) & 1;
                let vflip = (entry >> 11) & 1;
                let palette = (entry >> 12) & 0xF;
                print!(" {:3}", tile_num);
            }
        }
        println!();
    }

    // Check if screen block is all zeros
    let mut all_zero = true;
    let mut nonzero_entries = 0;
    for i in 0..1024 {
        let offset = bg0_screen_base + i * 2;
        if offset + 2 <= vram.len() {
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            if entry != 0 {
                all_zero = false;
                nonzero_entries += 1;
            }
        }
    }
    println!(
        "\nBG0 screen block: {} nonzero entries out of 1024",
        nonzero_entries
    );

    // Dump ALL unique tile numbers referenced by BG0 screen block
    let mut tile_set = std::collections::HashSet::new();
    for i in 0..1024 {
        let offset = bg0_screen_base + i * 2;
        if offset + 2 <= vram.len() {
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            let tile_num = entry & 0x3FF;
            if tile_num != 0 {
                tile_set.insert(tile_num);
            }
        }
    }
    let mut tiles: Vec<_> = tile_set.into_iter().collect();
    tiles.sort();
    println!("Unique tiles referenced: {:?}", tiles);

    // Check which of those tiles have data in VRAM
    let bg0_char_base = ((bg0cnt >> 2) & 3) as usize * 0x4000;
    for &tile in &tiles {
        let offset = bg0_char_base + tile as usize * 32;
        let nonzero = vram[offset..offset + 32]
            .iter()
            .filter(|&&b| b != 0)
            .count();
        if nonzero == 0 {
            println!("  Tile {}: NO DATA (all zeros)", tile);
        }
    }
}
