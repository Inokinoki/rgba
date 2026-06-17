use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Enable VRAM and DMA logging
    gba.mem.vram_log_enabled = true;
    gba.mem.dma_log_enabled = true;

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    let log_len = gba.mem.vram_write_log.len();
    let dma_len = gba.mem.dma_log.len();
    eprintln!("VRAM write log: {} entries", log_len);
    eprintln!("DMA log: {} entries", dma_len);

    // Analyze which tiles were written by CPU (not DMA)
    // Tile N is at VRAM offset N*32
    let mut tile_writes: std::collections::BTreeMap<u32, Vec<(u32, u8)>> =
        std::collections::BTreeMap::new();

    for &(addr, pc, val) in &gba.mem.vram_write_log {
        let raw_offset = (addr - 0x0600_0000) % 0x2_0000;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        let tile_id = offset / 32;
        tile_writes.entry(tile_id).or_default().push((pc, val));
    }

    // Find contiguous write ranges
    let tile_ids: Vec<u32> = tile_writes.keys().cloned().collect();
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for &tid in &tile_ids {
        if let Some(last) = ranges.last_mut() {
            if tid == last.1 + 1 {
                last.1 = tid;
            } else {
                ranges.push((tid, tid));
            }
        } else {
            ranges.push((tid, tid));
        }
    }
    eprintln!("\nCPU write ranges ({} tiles written):", tile_ids.len());
    for (start, end) in &ranges {
        eprintln!("  Tiles {}-{} ({} tiles)", start, end, end - start + 1);
    }

    // Check tiles 344-472 specifically
    eprintln!("\nTiles 344-472 CPU write status:");
    let mut any_written = false;
    for tid in 344..=472u32 {
        if let Some(writes) = tile_writes.get(&tid) {
            any_written = true;
            eprintln!(
                "  Tile {}: {} writes (first PC=0x{:08X})",
                tid,
                writes.len(),
                writes[0].0
            );
        }
    }
    if !any_written {
        eprintln!("  NO CPU WRITES to tiles 344-472!");
    }

    // DMA transfers to VRAM
    eprintln!("\nDMA transfers to VRAM:");
    let mut vram_dma_count = 0;
    for &(num, src, dst, count, size) in &gba.mem.dma_log {
        if dst >= 0x0600_0000 && dst < 0x0602_0000 {
            vram_dma_count += 1;
            if vram_dma_count <= 30 {
                eprintln!(
                    "  DMA{}: src=0x{:08X} dst=0x{:08X} count={} size={}",
                    num, src, dst, count, size
                );
            }
        }
    }
    if vram_dma_count > 30 {
        eprintln!("  ... and {} more", vram_dma_count - 30);
    }
    eprintln!("Total VRAM DMA transfers: {}", vram_dma_count);

    // Analyze DMA tile ranges
    let mut dma_tiles: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for &(num, src, dst, count, size) in &gba.mem.dma_log {
        if dst >= 0x0600_0000 && dst < 0x0602_0000 {
            let raw_offset = (dst - 0x0600_0000) % 0x2_0000;
            let offset = if raw_offset >= 0x1_8000 {
                raw_offset - 0x8000
            } else {
                raw_offset
            };
            for i in 0..count {
                let byte_off = offset + i * size;
                let tile = byte_off / 32;
                dma_tiles.insert(tile);
            }
        }
    }
    let mut dma_tile_vec: Vec<u32> = dma_tiles.into_iter().collect();
    dma_tile_vec.sort();
    let mut dma_ranges: Vec<(u32, u32)> = Vec::new();
    for &tid in &dma_tile_vec {
        if let Some(last) = dma_ranges.last_mut() {
            if tid == last.1 + 1 {
                last.1 = tid;
            } else {
                dma_ranges.push((tid, tid));
            }
        } else {
            dma_ranges.push((tid, tid));
        }
    }
    eprintln!("\nDMA write ranges ({} tiles):", dma_tile_vec.len());
    for (start, end) in &dma_ranges {
        eprintln!("  Tiles {}-{} ({} tiles)", start, end, end - start + 1);
    }

    // Final VRAM state
    gba.sync_ppu_full();
    let vram = gba.mem.vram();

    eprintln!("\nFinal tile content for tiles 340-480:");
    for tid in 340..480u32 {
        let off = tid as usize * 32;
        if off + 32 <= vram.len() {
            let nonzero = vram[off..off + 32].iter().any(|&b| b != 0);
            if nonzero {
                eprintln!("  Tile {}: HAS DATA", tid);
            }
        }
    }

    // Check BG screen entries
    let hofs = gba.ppu.get_bg_hofs(0);
    let vofs = gba.ppu.get_bg_vofs(0);
    eprintln!("\nBG0: hofs={} vofs={}", hofs, vofs);

    // Check which tiles BG3 (frontmost, priority=0) references
    eprintln!("\nBG3 (screen_base=0xF000) screen entries referencing tiles 344-472:");
    let mut count_in_range = 0;
    let mut count_total = 0;
    for row in 0..32 {
        for col in 0..32 {
            let entry_addr = 0xF000 + (row * 32 + col) * 2;
            if entry_addr + 1 < vram.len() {
                let entry = u16::from_le_bytes([vram[entry_addr], vram[entry_addr + 1]]);
                let tile = entry & 0x3FF;
                if tile >= 344 && tile <= 472 {
                    count_in_range += 1;
                }
                count_total += 1;
            }
        }
    }
    eprintln!(
        "  {}/{} entries reference tiles 344-472",
        count_in_range, count_total
    );

    // Same for BG0 (screen_base=0xC000)
    eprintln!("\nBG0 (screen_base=0xC000) screen entries referencing tiles 344-472:");
    count_in_range = 0;
    count_total = 0;
    for row in 0..32 {
        for col in 0..64 {
            let entry_addr = 0xC000 + (row * 64 + col) * 2;
            if entry_addr + 1 < vram.len() {
                let entry = u16::from_le_bytes([vram[entry_addr], vram[entry_addr + 1]]);
                let tile = entry & 0x3FF;
                if tile >= 344 && tile <= 472 {
                    count_in_range += 1;
                }
                count_total += 1;
            }
        }
    }
    eprintln!(
        "  {}/{} entries reference tiles 344-472",
        count_in_range, count_total
    );

    // PC page analysis for VRAM writes
    eprintln!("\nPC pages writing to VRAM:");
    let mut pc_pages: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for &(_, pc, _) in &gba.mem.vram_write_log {
        let page = pc & 0xFFFF_0000;
        *pc_pages.entry(page).or_insert(0) += 1;
    }
    for (page, count) in &pc_pages {
        eprintln!("  PC page 0x{:08X}: {} writes", page, count);
    }
}
