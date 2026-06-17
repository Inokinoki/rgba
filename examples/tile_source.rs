use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..200 {
        for _sl in 0..228 {
            gba.run_scanline();
        }
        if frame % 50 == 0 {
            eprintln!("Frame {}", frame);
        }
    }

    let log = &gba.mem().vram_write_log;
    eprintln!("Total VRAM writes: {}", log.len());

    let mut tile_writes: std::collections::BTreeMap<u32, Vec<(u32, u32)>> =
        std::collections::BTreeMap::new();

    for &(addr, pc, _val) in log {
        if addr >= 0x06000000 && addr < 0x0600C000 {
            let tile = (addr - 0x06000000) / 32;
            let tile_addr = tile * 32 + 0x06000000;
            tile_writes.entry(tile).or_default();
            let entry = tile_writes.get_mut(&tile).unwrap();
            if entry.len() < 5 {
                let already = entry.iter().any(|&(p, _)| p == pc);
                if !already {
                    entry.push((pc, addr));
                }
            }
        }
    }

    let nonzero_tiles: Vec<u32> = tile_writes.keys().cloned().collect();
    println!("Tiles with writes: {} tiles", nonzero_tiles.len());

    if !nonzero_tiles.is_empty() {
        println!(
            "First tile: {} (last PC: {:?})",
            nonzero_tiles[0], tile_writes[&nonzero_tiles[0]]
        );
        println!(
            "Last tile: {} (last PC: {:?})",
            nonzero_tiles[nonzero_tiles.len() - 1],
            tile_writes[&nonzero_tiles[nonzero_tiles.len() - 1]]
        );
    }

    let mut gaps: Vec<(u32, u32)> = Vec::new();
    let mut gap_start = 0u32;
    let mut in_gap = false;
    for t in 0..512u32 {
        if tile_writes.contains_key(&t) {
            if in_gap {
                gaps.push((gap_start, t - 1));
                in_gap = false;
            }
        } else {
            if !in_gap {
                gap_start = t;
                in_gap = true;
            }
        }
    }
    if in_gap {
        gaps.push((gap_start, 511));
    }

    println!("\nGaps in tile writes:");
    for (start, end) in &gaps {
        println!("  Tiles {}-{} ({} tiles)", start, end, end - start + 1);
    }

    println!("\nFirst 10 tiles with PCs:");
    for t in nonzero_tiles.iter().take(10) {
        println!("  Tile {}: {:?}", t, tile_writes[t]);
    }
    println!("\nTiles around 113-120:");
    for t in 110..130 {
        if tile_writes.contains_key(&t) {
            println!("  Tile {}: {:?}", t, tile_writes[&t]);
        } else {
            println!("  Tile {}: NO WRITES", t);
        }
    }
}
