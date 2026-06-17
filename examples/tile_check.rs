use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = gba.ppu();
    let vram = ppu.vram();
    let pal = gba.mem().palette();

    // Check tile 1023 data (4bpp, offset = 1023*32 = 0x7FE0)
    let tile_1023_offset = 1023 * 32;
    println!("=== Tile 1023 data (offset 0x{:05X}) ===", tile_1023_offset);
    let mut all_zero = true;
    for i in 0..32 {
        if vram[tile_1023_offset + i] != 0 {
            all_zero = false;
        }
    }
    println!("All zero: {}", all_zero);
    for row in 0..8 {
        print!("  Row {}: ", row);
        for col in 0..4 {
            let b = vram[tile_1023_offset + row * 4 + col];
            let lo = b & 0xF;
            let hi = b >> 4;
            print!("{}{} ", lo, hi);
        }
        println!();
    }

    // Check what palette 0 color 0 is
    let backdrop = u16::from_le_bytes([pal[0], pal[1]]);
    println!("\nBackdrop (pal[0]): {:04X}", backdrop);

    // Check last non-zero tile
    println!("\n=== Finding last tile with data ===");
    let mut last_nonzero = 0u16;
    for tn in 0..1024u16 {
        let base = tn as usize * 32;
        let mut has_data = false;
        for i in 0..32 {
            if vram[base + i] != 0 {
                has_data = true;
                break;
            }
        }
        if has_data {
            last_nonzero = tn;
        }
    }
    println!("Last tile with non-zero data: {}", last_nonzero);

    // Show tiles around the boundary
    println!("\n=== Tiles around last non-zero tile ===");
    for tn in (last_nonzero.saturating_sub(2))..=(last_nonzero + 2).min(1023) {
        let base = tn as usize * 32;
        let mut nonzero = 0;
        for i in 0..32 {
            if vram[base + i] != 0 {
                nonzero += 1;
            }
        }
        print!("  Tile {}: {} nonzero bytes, first 8: ", tn, nonzero);
        for i in 0..8.min(32) {
            print!("{:02X} ", vram[base + i]);
        }
        println!();
    }

    // Check screen entries for BG0 - show the pattern
    let bg0_screen = 0x0C000;
    println!("\n=== BG0 screen entries (first 4 rows) ===");
    for ty in 0..4 {
        print!("  Row {}: ", ty);
        for tx in 0..30 {
            let off = bg0_screen + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = entry & 0x3FF;
            let pal_num = (entry >> 12) & 0xF;
            print!("{:03x}{} ", tile_num, pal_num);
        }
        println!();
    }

    // Check if tiles 500-1023 have a pattern
    println!("\n=== Tile data presence by range ===");
    for range_start in (0..1024).step_by(64) {
        let range_end = (range_start + 64).min(1024);
        let mut nonzero_count = 0;
        for tn in range_start..range_end {
            let base = tn as usize * 32;
            let mut has_data = false;
            for i in 0..32 {
                if vram[base + i] != 0 {
                    has_data = true;
                    break;
                }
            }
            if has_data {
                nonzero_count += 1;
            }
        }
        println!(
            "  Tiles {:4}-{:-4}: {}/{} have data",
            range_start,
            range_end - 1,
            nonzero_count,
            range_end - range_start
        );
    }

    // Check VRAM 0x8000-0xC000 (between tile data and screen data)
    println!("\n=== VRAM 0x8000-0xC000 (should be OBJ tile area) ===");
    let mut nonzero = 0;
    for i in 0x8000..0xC000 {
        if vram[i] != 0 {
            nonzero += 1;
        }
    }
    println!(
        "Nonzero bytes: {}/16384 ({:.1}%)",
        nonzero,
        nonzero as f64 / 16384.0 * 100.0
    );

    // Check if VRAM has data in 0x10000+ (extended VRAM)
    let mut nonzero = 0;
    for i in 0x10000..0x18000 {
        if vram[i] != 0 {
            nonzero += 1;
        }
    }
    println!("VRAM 0x10000-0x18000: {} nonzero bytes", nonzero);

    // Count how many unique non-zero tile nums are in BG0 screen
    let mut unique_tiles: std::collections::HashSet<u16> = std::collections::HashSet::new();
    for ty in 0..32 {
        for tx in 0..32 {
            let off = bg0_screen + (ty * 32 + tx) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile_num = entry & 0x3FF;
            unique_tiles.insert(tile_num);
        }
    }
    println!("\nBG0 unique tile nums: {}", unique_tiles.len());
    let mut sorted_tiles: Vec<_> = unique_tiles.iter().collect();
    sorted_tiles.sort();
    print!("BG0 tile range: ");
    for t in sorted_tiles.iter().take(20) {
        print!("{} ", t);
    }
    println!("...");
}
