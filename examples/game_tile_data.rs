use rgba::Gba;

fn main() {
    let mut gba = Gba::new();

    // Increase VRAM log limit and trace
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Run only 10 frames to avoid log overflow
    for _ in 0..10 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();
    let write_log = &gba.mem.vram_write_log;

    println!("After 10 frames:");
    println!("VRAM write log: {} entries", write_log.len());

    // Analyze ALL writes to tile data area (offset 0x0000-0x7FFF)
    let mut tile_writes: std::collections::HashMap<u32, Vec<(u32, u8)>> =
        std::collections::HashMap::new();
    for &(addr, pc_encoded, val) in write_log {
        let offset = (addr & 0x1FFFF) as usize;
        if offset < 0x8000 {
            let tile = (offset / 32) as u32;
            tile_writes
                .entry(tile)
                .or_default()
                .push(((pc_encoded << 1), val));
        }
    }

    let mut tiles: Vec<u32> = tile_writes.keys().copied().collect();
    tiles.sort();

    println!("\nTiles with logged writes: {}", tiles.len());
    for &tile in &tiles {
        let writes = &tile_writes[&tile];
        let non_zero: Vec<_> = writes.iter().filter(|(_, v)| *v != 0).collect();

        // Check tile data in VRAM
        let tile_off = (tile as usize) * 32;
        let mut vram_sum = 0u32;
        for i in 0..32 {
            vram_sum += vram[tile_off + i] as u32;
        }

        println!(
            "  Tile {}: {} writes ({} non-zero), VRAM sum={}",
            tile,
            writes.len(),
            non_zero.len(),
            vram_sum
        );

        // For tiles near 100, show actual data
        if tile >= 95 && tile <= 120 {
            print!("    VRAM bytes: ");
            for i in 0..32 {
                print!("{:02X}", vram[tile_off + i]);
            }
            println!();
        }
    }

    // Check tiles 0-150: which have data in VRAM?
    println!("\n=== Tiles 0-150 with data ===");
    for tile in 0u32..150 {
        let off = (tile as usize) * 32;
        let mut sum = 0u32;
        for i in 0..32 {
            sum += vram[off + i] as u32;
        }
        if sum > 0 {
            let has_writes = tile_writes.contains_key(&tile);
            println!(
                "  Tile {}: VRAM sum={} (writes logged: {})",
                tile, sum, has_writes
            );
        }
    }

    // Check where the STRH at 0x080D0BFA writes
    println!("\n=== VRAM writes from PC 0x080D0BFA ===");
    let mut target_addrs: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for &(addr, pc_encoded, val) in write_log {
        let pc = pc_encoded << 1;
        if pc == 0x080D0BFA || pc == 0x080D0BEA {
            let offset = addr & 0x1FFFF;
            if offset < 0x8000 {
                target_addrs.insert(offset);
            }
        }
    }
    let mut sorted: Vec<u32> = target_addrs.into_iter().collect();
    sorted.sort();
    println!("VRAM offsets written: {} unique", sorted.len());
    if sorted.len() <= 50 {
        for addr in &sorted {
            let tile = addr / 32;
            let byte_in_tile = addr % 32;
            println!(
                "  Offset {:#06X} (tile {} byte {})",
                addr, tile, byte_in_tile
            );
        }
    } else {
        println!(
            "Range: {:#06X}-{:#06X}",
            sorted[0],
            sorted[sorted.len() - 1]
        );
        // Count by tile
        let mut tile_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
        for &addr in &sorted {
            let tile = addr / 32;
            *tile_counts.entry(tile).or_insert(0) += 1;
        }
        let mut tc: Vec<_> = tile_counts.into_iter().collect();
        tc.sort_by_key(|&(t, _)| t);
        for (tile, count) in &tc {
            println!("  Tile {}: {} unique byte offsets", tile, count);
        }
    }
}
