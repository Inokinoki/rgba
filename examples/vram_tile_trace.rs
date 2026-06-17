use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for frame in 0..300 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    let log = &gba.mem().vram_write_log;
    println!("Total VRAM writes logged: {}", log.len());

    // The log stores raw addresses (0x06000000+), not offsets
    // BG VRAM: 0x06000000-0x0600FFFF
    // OBJ VRAM: 0x06010000-0x06017FFF
    let bg_writes: Vec<_> = log
        .iter()
        .filter(|(addr, _, _)| *addr >= 0x06000000 && *addr < 0x06010000)
        .collect();
    let obj_writes: Vec<_> = log
        .iter()
        .filter(|(addr, _, _)| *addr >= 0x06010000 && *addr < 0x06018000)
        .collect();
    println!("BG VRAM writes: {}", bg_writes.len());
    println!("OBJ VRAM writes: {}", obj_writes.len());

    // Count writes per 32-byte tile in BG VRAM
    let mut tile_writes = vec![0u32; 2048];
    for &(addr, _pc, _val) in log {
        let offset = (addr - 0x06000000) as usize;
        if offset < 0x10000 {
            let tile_idx = offset / 32;
            if tile_idx < 2048 {
                tile_writes[tile_idx] += 1;
            }
        }
    }

    let tiles_written: usize = tile_writes.iter().filter(|&&c| c > 0).count();
    println!("Unique BG tiles written: {}", tiles_written);

    // Show tile write ranges
    println!("\n=== BG tile write ranges ===");
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut in_range = false;
    let mut range_start = 0;
    for (i, &count) in tile_writes.iter().enumerate() {
        if count > 0 && !in_range {
            range_start = i;
            in_range = true;
        } else if count == 0 && in_range {
            ranges.push((range_start, i - 1));
            in_range = false;
        }
    }
    if in_range {
        ranges.push((range_start, 2047));
    }
    for (start, end) in &ranges {
        println!("  Tiles {}-{}", start, end);
    }

    // Show sample VRAM content
    println!("\n=== VRAM tile content samples ===");
    let vram = gba.mem().vram();
    for tile in [0, 1, 50, 100, 113, 114, 200, 394, 400, 450, 500, 511] {
        let offset = tile * 32;
        if offset + 32 <= vram.len() {
            let nonzero: usize = vram[offset..offset + 32]
                .iter()
                .filter(|&&b| b != 0)
                .count();
            print!("  Tile {}: {} nonzero bytes", tile, nonzero);
            if nonzero > 0 {
                print!("  first: {:02x?}", &vram[offset..offset + 8]);
            }
            println!();
        }
    }

    // Check char block boundaries
    println!("\n=== Char block analysis ===");
    for cb in 0..4 {
        let base = cb * 0x4000;
        let mut nonzero = 0;
        let mut total = 0;
        for tile in 0..128 {
            let off = base + tile * 32;
            if off + 32 <= vram.len() {
                let has = vram[off..off + 32].iter().any(|&b| b != 0);
                if has {
                    nonzero += 1;
                }
                total += 1;
            }
        }
        println!(
            "  Char block {} (0x{:05X}-0x{:05X}): {}/{} nonzero tiles",
            cb,
            base,
            base + 0x3FFF,
            nonzero,
            total
        );
    }
}
