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

    let ppu = gba.ppu();
    let vram = ppu.vram();

    // For BG3, check tiles that appear in the screen map
    let bgcnt = ppu.get_bgcnt(3);
    let map_base = ppu.get_bg_map_base(3) as usize;
    let tile_base = ppu.get_bg_tile_base(3) as usize;
    let is_8bpp = (bgcnt & 0x80) != 0;

    // Get all unique tile numbers from BG3 screen map
    let mut tile_set = std::collections::HashSet::new();
    for i in 0..2048 {
        let off = map_base + i * 2;
        if off + 1 < vram.len() {
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile = entry & 0x3FF;
            tile_set.insert(tile);
        }
    }

    eprintln!(
        "BG3: {} unique tiles in screen map (tile_base=0x{:05X}, 8bpp={})",
        tile_set.len(),
        tile_base,
        is_8bpp
    );

    // Check tile data for each unique tile
    let bytes_per_tile = if is_8bpp { 64 } else { 32 };
    let mut all_zero_tiles = 0;
    let mut data_tiles = 0;
    for tile in &tile_set {
        let off = tile_base + *tile as usize * bytes_per_tile;
        let mut all_zero = true;
        if off + bytes_per_tile <= vram.len() {
            for b in 0..bytes_per_tile {
                if vram[off + b] != 0 {
                    all_zero = false;
                    break;
                }
            }
        }
        if all_zero {
            all_zero_tiles += 1;
        } else {
            data_tiles += 1;
        }
    }
    eprintln!("  All-zero tiles: {}/{}", all_zero_tiles, tile_set.len());
    eprintln!("  Tiles with data: {}/{}", data_tiles, tile_set.len());

    // Render a few data tiles as ASCII art
    let data_tile_list: Vec<u16> = tile_set
        .iter()
        .filter(|t| {
            let off = tile_base + **t as usize * bytes_per_tile;
            if off + bytes_per_tile > vram.len() {
                return false;
            }
            for b in 0..bytes_per_tile {
                if vram[off + b] != 0 {
                    return true;
                }
            }
            false
        })
        .copied()
        .collect();

    for tile in data_tile_list.iter().take(5) {
        let off = tile_base + *tile as usize * bytes_per_tile;
        eprintln!("\nTile {} (offset 0x{:05X}):", tile, off);
        for row in 0..8 {
            let mut line = String::new();
            if is_8bpp {
                for col in 0..8 {
                    let pixel = vram[off + row * 8 + col];
                    let c = if pixel == 0 {
                        '.'
                    } else if pixel < 16 {
                        '0'
                    } else {
                        'X'
                    };
                    line.push(c);
                }
            } else {
                for col in 0..4 {
                    let byte = vram[off + row * 4 + col];
                    let lo = byte & 0xF;
                    let hi = (byte >> 4) & 0xF;
                    let cl = if lo == 0 {
                        '.'
                    } else {
                        std::char::from_digit(lo as u32, 16).unwrap()
                    };
                    let ch = if hi == 0 {
                        '.'
                    } else {
                        std::char::from_digit(hi as u32, 16).unwrap()
                    };
                    line.push(cl);
                    line.push(ch);
                }
            }
            eprintln!("  {}", line);
        }
    }

    // Check tile 1023
    let tile_1023_off = tile_base + 1023 * bytes_per_tile;
    eprintln!("\nTile 1023 (offset 0x{:05X}):", tile_1023_off);
    if tile_1023_off + bytes_per_tile <= vram.len() {
        let mut all_ff = true;
        for b in 0..bytes_per_tile {
            if vram[tile_1023_off + b] != 0xFF {
                all_ff = false;
                break;
            }
        }
        eprintln!("  All 0xFF: {}", all_ff);
    }

    // Check if VRAM char base 0 area is being used for something else
    // Screen maps overlap with tile data: maps are at 0xC000-0xFFFF
    // In GBA, screen maps and tile data share VRAM but shouldn't overlap
    // char_base 0: 0x00000-0x0BFFF (48KB = tiles 0-1023 for 4bpp or 0-511 for 8bpp)
    // screen maps: 0x0C000-0x0FFFF (16KB = 8 screen blocks of 2KB each)
    // This means tile data 0-1023 uses 0x0000-0x7FFF (32KB), and screen maps use 0x8000+
    // Wait, 1024 tiles * 32 bytes = 32768 = 0x8000 bytes for 4bpp
    // And char_base 0 means tile data starts at 0x0000
    // Screen block base 24 (0xC000/0x800=24) is separate from tile data

    // Actually, in GBA, the full VRAM layout is:
    // Char base 0: 0x00000-0x0FFFF (4 blocks of 16KB)
    // Screen base: interleaved - screen blocks are at various offsets
    // The important thing: char base 0 covers 0x00000-0x0FFFF
    // Screen block at 0x0C000 is WITHIN char base 0 range!
    // So tile 1023 (at offset 1023*32 = 0x7FE0) is within the first 32KB
    // Screen maps at 0xC000+ are in the second half of char base 0

    eprintln!("\nVRAM layout check:");
    eprintln!("  Char base 0 range: 0x00000-0x0FFFF (4 * 16KB)");
    eprintln!("  Tile 1023 4bpp offset: 0x{:05X}", 1023 * 32);
    eprintln!("  Screen maps start at: 0x{:05X}", 0xC000);
    eprintln!("  Tiles 0-1023 (4bpp) end at: 0x{:05X}", 1024 * 32);
    eprintln!("  => Tiles 0-1023 fit in 0x00000-0x07FFF");
    eprintln!("  => Screen maps at 0x0C000 don't overlap with tile data");
}
