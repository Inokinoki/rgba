use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    for _ in 0..3000 {
        gba.run_frame();
    }
    gba.sync_ppu_full();
    gba.sync_ppu();

    let vram = gba.ppu().vram();

    // Check tile 1023 (4bpp = 32 bytes per tile)
    let tile_1023_offset = 1023 * 32;
    eprintln!("Tile 1023 (4bpp) at offset 0x{:05X}:", tile_1023_offset);
    let mut all_ff = true;
    for b in 0..32 {
        if vram[tile_1023_offset + b] != 0xFF {
            all_ff = false;
        }
    }
    eprintln!("  All 0xFF: {}", all_ff);
    if !all_ff {
        for row in 0..8 {
            let off = tile_1023_offset + row * 4;
            eprintln!(
                "    row {}: {:02X} {:02X} {:02X} {:02X}",
                row,
                vram[off],
                vram[off + 1],
                vram[off + 2],
                vram[off + 3]
            );
        }
    }

    // Check non-zero tile data in VRAM
    let mut nonzero_tiles = 0;
    let mut data_tiles = Vec::new();
    for tile in 0..1024 {
        let off = tile * 32;
        let mut has_data = false;
        let mut all_same = true;
        let first = vram[off];
        for b in 0..32 {
            if vram[off + b] != 0 {
                has_data = true;
            }
            if vram[off + b] != first {
                all_same = false;
            }
        }
        if has_data && !all_same {
            nonzero_tiles += 1;
            if data_tiles.len() < 10 {
                data_tiles.push(tile);
            }
        }
    }
    eprintln!(
        "\nNon-trivial tiles (not all-zero, not all-same): {}/1024",
        nonzero_tiles
    );

    // Sample a few data tiles
    for tile in &data_tiles {
        let off = *tile as usize * 32;
        eprintln!("Tile {} at 0x{:05X}:", tile, off);
        for row in 0..8 {
            let r = off + row * 4;
            eprintln!(
                "  row {}: {:02X} {:02X} {:02X} {:02X}",
                row,
                vram[r],
                vram[r + 1],
                vram[r + 2],
                vram[r + 3]
            );
        }
    }

    // Check VRAM regions used by screen maps
    // BG0 map at 0xC000, BG1 at 0xD000, BG2 at 0xE000, BG3 at 0xF000
    // Each map is 0x800 bytes for 32x32 tiles (2048 bytes for full 1024 entries)
    // But with size=1 (512x256), need 2 screen blocks = 0x1000 bytes
    for (name, base) in [
        ("BG0", 0xC000),
        ("BG1", 0xD000),
        ("BG2", 0xE000),
        ("BG3", 0xF000),
    ] {
        let mut unique = std::collections::HashSet::new();
        let mut non_1023 = 0;
        for i in 0..1024 {
            let off = base + i * 2;
            if off + 1 < vram.len() {
                let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
                let tile = entry & 0x3FF;
                unique.insert(tile);
                if tile != 1023 {
                    non_1023 += 1;
                }
            }
        }
        eprintln!(
            "\n{} screen map at 0x{:05X}: {} unique tiles, {} non-1023 entries",
            name,
            base,
            unique.len(),
            non_1023
        );

        // Show which rows have non-1023 tiles
        let mut first_non_1023_row = None;
        let mut last_non_1023_row = None;
        for row in 0..32 {
            let mut row_has_data = false;
            for col in 0..32 {
                let off = base + (row * 32 + col) * 2;
                if off + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
                    if (entry & 0x3FF) != 1023 {
                        row_has_data = true;
                    }
                }
            }
            if row_has_data {
                if first_non_1023_row.is_none() {
                    first_non_1023_row = Some(row);
                }
                last_non_1023_row = Some(row);
            }
        }
        if let (Some(first), Some(last)) = (first_non_1023_row, last_non_1023_row) {
            eprintln!("  Non-1023 rows: {} to {}", first, last);
        }
    }

    // Check if VRAM in tile data area (0x0000-0xBFFF) is actually being populated
    let mut vram_nonzero = 0;
    let mut tile_area_nonzero = 0;
    for i in 0..vram.len() {
        if vram[i] != 0 {
            vram_nonzero += 1;
        }
    }
    for i in 0..0xC000 {
        if vram[i] != 0 {
            tile_area_nonzero += 1;
        }
    }
    eprintln!(
        "\nVRAM total: {} nonzero out of {} ({:.1}%)",
        vram_nonzero,
        vram.len(),
        vram_nonzero as f64 / vram.len() as f64 * 100.0
    );
    eprintln!(
        "VRAM tile area (0-0xBFFF): {} nonzero out of {} ({:.1}%)",
        tile_area_nonzero,
        0xC000,
        tile_area_nonzero as f64 / 0xC000 as f64 * 100.0
    );
}
