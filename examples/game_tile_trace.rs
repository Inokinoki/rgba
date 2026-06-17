use rgba::Gba;
use std::collections::{HashMap, HashSet};

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.mem.dma_log_enabled = true;
    gba.mem.swi_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..300 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();
    let write_log = &gba.mem.vram_write_log;
    let dma_log = &gba.mem.dma_log;

    println!("=== After 300 frames ===");
    println!("VRAM write log: {} entries", write_log.len());
    println!("DMA log: {} entries", dma_log.len());
    println!("SWI log: {} entries", gba.mem.swi_log.len());

    // Find all tiles with data
    let mut tiles_with_data: Vec<u32> = Vec::new();
    let mut max_tile = 0u32;
    for t in 0..1024 {
        let off = t * 32;
        if off + 32 <= vram.len() && vram[off..off + 32].iter().any(|&b| b != 0) {
            tiles_with_data.push(t as u32);
            max_tile = t as u32;
        }
    }
    println!(
        "\nTiles with data: {} (max={})",
        tiles_with_data.len(),
        max_tile
    );

    // Find gaps in tile data
    if !tiles_with_data.is_empty() {
        let mut gaps = Vec::new();
        let mut prev = tiles_with_data[0];
        for &t in &tiles_with_data[1..] {
            if t > prev + 1 {
                gaps.push((prev + 1, t - 1));
            }
            prev = t;
        }
        if !gaps.is_empty() {
            println!("Gaps in tile data:");
            for (start, end) in &gaps {
                println!("  Tiles {}-{}", start, end);
            }
        }

        // Print ranges of consecutive tiles with data
        let mut ranges = Vec::new();
        let mut range_start = tiles_with_data[0];
        let mut range_end = tiles_with_data[0];
        for &t in &tiles_with_data[1..] {
            if t == range_end + 1 {
                range_end = t;
            } else {
                ranges.push((range_start, range_end));
                range_start = t;
                range_end = t;
            }
        }
        ranges.push((range_start, range_end));
        println!("Consecutive tile ranges with data:");
        for (s, e) in &ranges {
            println!("  Tiles {}-{} ({} tiles)", s, e, e - s + 1);
        }
    }

    // Analyze VRAM writes to tile data area (offset 0x0000-0x7FFF)
    let tile_area_end = 1024 * 32; // 0x8000
    let mut tile_writes_by_pc: HashMap<u32, Vec<(u32, u8)>> = HashMap::new();
    let mut tile_writes_by_tile: HashMap<u32, Vec<(u32, u8)>> = HashMap::new();

    for &(addr, pc_encoded, val) in write_log {
        let offset = (addr & 0x1FFFF) as usize;
        if offset < tile_area_end {
            let pc = pc_encoded << 1;
            tile_writes_by_pc.entry(pc).or_default().push((addr, val));
            let tile_num = (offset / 32) as u32;
            tile_writes_by_tile
                .entry(tile_num)
                .or_default()
                .push((addr, val));
        }
    }

    println!("\n=== Tile data area writes by source PC ===");
    let mut pc_list: Vec<_> = tile_writes_by_pc.iter().collect();
    pc_list.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    for (pc, writes) in pc_list.iter().take(20) {
        let min_addr = writes.iter().map(|&(a, _)| a).min().unwrap();
        let max_addr = writes.iter().map(|&(a, _)| a).max().unwrap();
        let min_tile = ((min_addr & 0x1FFFF) as usize / 32) as u32;
        let max_tile = ((max_addr & 0x1FFFF) as usize / 32) as u32;
        println!(
            "  PC={:#010X}: {} writes, tiles {}-{}",
            pc,
            writes.len(),
            min_tile,
            max_tile
        );
    }

    // Check which tiles received writes
    println!("\n=== Tiles that received VRAM writes ===");
    let mut written_tiles: Vec<u32> = tile_writes_by_tile.keys().copied().collect();
    written_tiles.sort();
    if !written_tiles.is_empty() {
        let mut wranges = Vec::new();
        let mut ws = written_tiles[0];
        let mut we = written_tiles[0];
        for &t in &written_tiles[1..] {
            if t == we + 1 {
                we = t;
            } else {
                wranges.push((ws, we));
                ws = t;
                we = t;
            }
        }
        wranges.push((ws, we));
        for (s, e) in &wranges {
            println!("  Tiles {}-{} ({} tiles)", s, e, e - s + 1);
        }
    }

    // Tiles with data but NO writes in log (loaded before logging started or via DMA)
    let written_set: HashSet<u32> = written_tiles.iter().copied().collect();
    let data_set: HashSet<u32> = tiles_with_data.iter().copied().collect();
    let data_no_writes: Vec<u32> = data_set.difference(&written_set).copied().collect();
    if !data_no_writes.is_empty() {
        println!("\nTiles with data but no logged writes (loaded before logging?):");
        let mut ranges = Vec::new();
        let mut sorted = data_no_writes.clone();
        sorted.sort();
        let mut rs = sorted[0];
        let mut re = sorted[0];
        for &t in &sorted[1..] {
            if t == re + 1 {
                re = t;
            } else {
                ranges.push((rs, re));
                rs = t;
                re = t;
            }
        }
        ranges.push((rs, re));
        for (s, e) in &ranges {
            println!("  Tiles {}-{} ({} tiles)", s, e, e - s + 1);
        }
    }

    // DMA analysis: which DMA transfers target tile data area
    println!("\n=== DMA transfers to VRAM tile area (offset 0x0000-0x7FFF) ===");
    let mut tile_dma_count = 0;
    for &(ch, src, dst, count, size) in dma_log {
        if dst >= 0x0600_0000 && dst < 0x0601_8000 {
            let off = (dst - 0x0600_0000) as usize;
            if off < tile_area_end {
                tile_dma_count += 1;
                if tile_dma_count <= 20 {
                    let dst_tile = off / 32;
                    println!(
                        "  DMA{}: {:#010X}→{:#010X} (tile {}) count={} size={}",
                        ch, src, dst, dst_tile, count, size
                    );
                }
            }
        }
    }
    println!("Total DMA to tile area: {}", tile_dma_count);

    // SWI analysis
    println!("\n=== SWI calls ===");
    let mut swi_counts: HashMap<u32, u32> = HashMap::new();
    for &swi_num in &gba.mem.swi_log {
        *swi_counts.entry(swi_num).or_insert(0) += 1;
    }
    let mut swi_sorted: Vec<_> = swi_counts.iter().collect();
    swi_sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (&num, &count) in &swi_sorted {
        let name = match num {
            0x01 => "RegisterRamReset",
            0x04 => "IntrWait",
            0x06 => "Div",
            0x0B => "CpuSet",
            _ => "Unknown",
        };
        println!("  SWI {:02X} ({}): {} calls", num, name, count);
    }

    // Check tile 1023 specifically (the opaque white tile)
    println!("\n=== Tile 1023 analysis ===");
    let t1023_off = 1023 * 32;
    let t1023_data = &vram[t1023_off..t1023_off + 32];
    let all_ff = t1023_data.iter().all(|&b| b == 0xFF);
    println!("Tile 1023 all 0xFF: {}", all_ff);
    if tile_writes_by_tile.contains_key(&1023) {
        let writes = &tile_writes_by_tile[&1023];
        println!("Tile 1023 had {} writes", writes.len());
    } else {
        println!("Tile 1023: no logged writes (filled before logging started)");
    }

    // Key question: does tile 187+ area receive ANY writes at all?
    println!("\n=== Critical: writes to tile 187+ area ===");
    let mut writes_beyond_187 = 0;
    let mut pcs_beyond_187: HashSet<u32> = HashSet::new();
    for &(addr, pc_encoded, _val) in write_log {
        let offset = (addr & 0x1FFFF) as usize;
        if offset >= 187 * 32 && offset < tile_area_end {
            writes_beyond_187 += 1;
            pcs_beyond_187.insert(pc_encoded << 1);
        }
    }
    println!("Writes to tiles 187+ area: {}", writes_beyond_187);
    if !pcs_beyond_187.is_empty() {
        println!("Source PCs:");
        for pc in &pcs_beyond_187 {
            println!("  {:#010X}", pc);
        }
    }
}
