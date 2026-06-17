use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    for _ in 0..500 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let vram = gba.ppu().vram();

    // Tile 1023 at offset 0x7FE0
    let tile_off = 1023 * 32;
    println!("Tile 1023 data (offset 0x{:04X}):", tile_off);
    for row in 0..8 {
        let off = tile_off + row * 4;
        let mut pixels = String::new();
        for byte in 0..4 {
            let b = vram[off + byte];
            let lo = b & 0xF;
            let hi = b >> 4;
            pixels.push_str(&format!("{}{} ", lo, hi));
        }
        println!("  Row {}: {}", row, pixels);
    }

    // All screen entries for BG3 are tile 1023
    // This means BG3 is filled with a repeating pattern of tile 1023
    // If tile 1023 has non-zero color indices, BG3 shows content (not transparent)
    // If all are 0xF (white), that explains the white screen

    // Let's also check what ALL BG screen entries look like
    for bg in 0..4 {
        let screen_base = gba.ppu().get_bg_map_base(bg) as usize;
        let mut tile_counts: std::collections::HashMap<u16, u32> = std::collections::HashMap::new();
        for i in 0..1024 {
            let off = screen_base + i * 2;
            if off + 1 < vram.len() {
                let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
                let tile = e & 0x3FF;
                *tile_counts.entry(tile).or_insert(0) += 1;
            }
        }
        let mut entries: Vec<_> = tile_counts.into_iter().collect();
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        println!("\nBG{} screen entries (top 5 tiles):", bg);
        for (tile, count) in entries.iter().take(5) {
            println!("  tile {} appears {} times", tile, count);
        }
    }
}
