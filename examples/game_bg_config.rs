use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..600 {
        gba.run_frame();

        if frame == 10 || frame == 50 || frame == 100 || frame == 300 || frame == 599 {
            // Read all registers first
            let dispcnt = gba.mem.read_half(0x04000000);
            let bg0cnt = gba.mem.read_half(0x04000008);
            let bg1cnt = gba.mem.read_half(0x0400000A);
            let bg2cnt = gba.mem.read_half(0x0400000C);
            let bg3cnt = gba.mem.read_half(0x0400000E);
            let bg0hofs = gba.mem.read_half(0x04000010) & 0x1FF;
            let bg0vofs = gba.mem.read_half(0x04000012) & 0x1FF;
            let bg1hofs = gba.mem.read_half(0x04000014) & 0x1FF;
            let bg1vofs = gba.mem.read_half(0x04000016) & 0x1FF;
            let bg2hofs = gba.mem.read_half(0x04000018) & 0x1FF;
            let bg2vofs = gba.mem.read_half(0x0400001A) & 0x1FF;
            let bg3hofs = gba.mem.read_half(0x0400001C) & 0x1FF;
            let bg3vofs = gba.mem.read_half(0x0400001E) & 0x1FF;

            // Clone VRAM data to avoid borrow issues
            let vram = gba.mem().vram().to_vec();

            println!("\n=== Frame {} ===", frame);
            println!(
                "DISPCNT: {:#06X} (mode={}, BG={:#05b}, OBJ={})",
                dispcnt,
                dispcnt & 7,
                (dispcnt >> 8) & 0xF,
                (dispcnt >> 12) & 1
            );

            for (i, cnt) in [bg0cnt, bg1cnt, bg2cnt, bg3cnt].iter().enumerate() {
                let priority = cnt & 3;
                let char_base = (cnt >> 2) & 3;
                let mosaic = (cnt >> 6) & 1;
                let color_mode = (cnt >> 7) & 1;
                let screen_base = (cnt >> 8) & 0x1F;
                let screen_size = (cnt >> 14) & 3;

                let char_addr = char_base * 0x4000;
                let screen_addr = screen_base * 0x800;

                println!("BG{}CNT: {:#06X} pri={} char_base={:#06X} scr_base={:#06X} {}bit mosaic={} size={}",
                    i, cnt, priority, char_addr, screen_addr,
                    if color_mode == 1 { "8" } else { "4" },
                    mosaic, screen_size);
            }

            println!("BG0 offset: ({}, {})", bg0hofs, bg0vofs);
            println!("BG1 offset: ({}, {})", bg1hofs, bg1vofs);
            println!("BG2 offset: ({}, {})", bg2hofs, bg2vofs);
            println!("BG3 offset: ({}, {})", bg3hofs, bg3vofs);

            // Check tile data at each character base
            for cb in 0..4 {
                let base = cb * 0x4000;
                let mut has_data = 0u32;
                let mut first_data_tile = 9999usize;
                let mut last_data_tile = 0usize;
                for t in 0..512 {
                    let off = base + t * 32;
                    if off + 32 <= vram.len() && vram[off..off + 32].iter().any(|&b| b != 0) {
                        has_data += 1;
                        first_data_tile = first_data_tile.min(t);
                        last_data_tile = last_data_tile.max(t);
                    }
                }
                if has_data > 0 {
                    println!(
                        "Char base {} ({:#06X}): {} tiles with data, range {}-{}",
                        cb, base, has_data, first_data_tile, last_data_tile
                    );
                }
            }

            // Check screen entries for BG layers
            for (bg_idx, cnt) in [bg0cnt, bg1cnt, bg2cnt, bg3cnt].iter().enumerate() {
                let screen_base = ((*cnt >> 8) & 0x1F) as usize * 0x800;
                let screen_size = (*cnt >> 14) & 3;
                let char_base = ((*cnt >> 2) & 3) as usize * 0x4000;
                let color_mode = (*cnt >> 7) & 1;

                let entries_per_screen = match screen_size {
                    0 => 1024,
                    1 => 2048,
                    2 => 2048,
                    3 => 4096,
                    _ => 0,
                };

                let mut nonzero = 0u32;
                let mut tiles = std::collections::HashSet::new();
                for i in 0..entries_per_screen {
                    let off = screen_base + i * 2;
                    if off + 2 <= vram.len() {
                        let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
                        if entry != 0 {
                            nonzero += 1;
                            tiles.insert(entry & 0x3FF);
                        }
                    }
                }

                if nonzero > 0 {
                    let mut tile_list: Vec<u16> = tiles.into_iter().collect();
                    tile_list.sort();
                    let mut empty_tiles = 0u32;
                    let mut non_empty_tiles = 0u32;
                    for &t in &tile_list {
                        let tile_bytes = if color_mode == 1 {
                            char_base + (t as usize) * 64
                        } else {
                            char_base + (t as usize) * 32
                        };
                        if tile_bytes + (if color_mode == 1 { 64 } else { 32 }) <= vram.len() {
                            let len = if color_mode == 1 { 64 } else { 32 };
                            if vram[tile_bytes..tile_bytes + len].iter().any(|&b| b != 0) {
                                non_empty_tiles += 1;
                            } else {
                                empty_tiles += 1;
                            }
                        }
                    }
                    println!("BG{} screen: {} non-zero entries, {} unique tiles ({} have data, {} empty)",
                        bg_idx, nonzero, tile_list.len(), non_empty_tiles, empty_tiles);
                    if tile_list.len() <= 30 {
                        print!("  Tiles:");
                        for t in &tile_list {
                            let tile_bytes = char_base + (*t as usize) * 32;
                            let has = tile_bytes + 32 <= vram.len()
                                && vram[tile_bytes..tile_bytes + 32].iter().any(|&b| b != 0);
                            print!(" {}{}", t, if has { "" } else { "!" });
                        }
                        println!();
                    } else {
                        print!(
                            "  Tile range: {}-{}",
                            tile_list.first().unwrap(),
                            tile_list.last().unwrap()
                        );
                        // Count how many in first 50
                        let first50 = tile_list
                            .iter()
                            .take(50)
                            .filter(|&&t| {
                                let tb = char_base + (t as usize) * 32;
                                tb + 32 <= vram.len() && vram[tb..tb + 32].iter().any(|&b| b != 0)
                            })
                            .count();
                        println!(" (first 50: {} have data)", first50);
                    }
                }
            }
        }
    }
}
