use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.vram_log_enabled = true;

    // Run until we have some data
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check what values were written to tiles 344-472 area
    // Only look at first 1000 entries (earliest writes)
    let log = &gba.mem.vram_write_log;

    // Find first non-zero write to tiles 344-472
    let tile_range_start: usize = 344 * 32;
    let tile_range_end: usize = 473 * 32;

    let mut first_nonzero_write: Option<(usize, u32, u32, u8)> = None;
    let mut zero_writes = 0usize;
    let mut nonzero_writes = 0usize;

    for (i, &(addr, pc, val)) in log.iter().enumerate() {
        let raw_offset = ((addr - 0x0600_0000) % 0x2_0000) as usize;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        if offset >= tile_range_start && offset < tile_range_end {
            if val != 0 {
                nonzero_writes += 1;
                if first_nonzero_write.is_none() {
                    first_nonzero_write = Some((i, addr, pc, val));
                }
            } else {
                zero_writes += 1;
            }
        }
    }

    eprintln!(
        "Writes to tiles 344-472: {} zero, {} non-zero",
        zero_writes, nonzero_writes
    );
    if let Some((i, addr, pc, val)) = first_nonzero_write {
        eprintln!(
            "First non-zero write: entry #{}, addr=0x{:08X}, pc=0x{:08X}, val=0x{:02X}",
            i, addr, pc, val
        );
    }

    // The log is 100K but may be full - check what PC pages are writing
    // and check what tiles 0-343 look like (they have data)
    // Are tiles 0-343 written by same PC?
    eprintln!("\nChecking tiles 0-343 (which have data):");
    let mut pc_for_data_tiles: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
    for &(_, pc, val) in log.iter() {
        if val != 0 {
            pc_for_data_tiles.insert(pc);
        }
    }
    eprintln!("PCs that write non-zero values:");
    for pc in &pc_for_data_tiles {
        eprintln!("  PC=0x{:08X}", pc);
    }

    // Check: is the decompression function writing to the WRONG addresses?
    // Let's look at the addresses that PC=0x080D0BFA writes to
    eprintln!("\nAddress ranges written by PC=0x080D0BFA:");
    let mut addrs: Vec<u32> = log
        .iter()
        .filter(|&&(_, pc, _)| pc == 0x080D0BFA)
        .map(|&(addr, _, _)| addr)
        .collect();
    addrs.sort();
    addrs.dedup();
    if !addrs.is_empty() {
        eprintln!(
            "  First addr: 0x{:08X}, last: 0x{:08X}, count: {}",
            addrs[0],
            addrs[addrs.len() - 1],
            addrs.len()
        );
    }

    // Check all unique PCs writing non-zero to VRAM
    let mut nonzero_pc_counts: std::collections::BTreeMap<u32, (usize, usize)> =
        std::collections::BTreeMap::new();
    for &(_, pc, val) in log.iter() {
        let entry = nonzero_pc_counts.entry(pc).or_insert((0, 0));
        entry.0 += 1;
        if val != 0 {
            entry.1 += 1;
        }
    }
    eprintln!("\nPC write summary (total, nonzero):");
    for (pc, (total, nz)) in &nonzero_pc_counts {
        eprintln!("  PC=0x{:08X}: {} total, {} nonzero", pc, total, nz);
    }

    // KEY QUESTION: is the decompression actually writing zeros to tiles 344-472?
    // Or is something clearing them AFTER they're written?
    // Let's check the ORDER of writes to tile 394 (a known empty tile)
    eprintln!(
        "\nAll writes to tile 394 (offset 0x{:04X}-0x{:04X}):",
        394 * 32,
        395 * 32 - 1
    );
    let mut tile_394_writes: Vec<(usize, u32, u8)> = Vec::new();
    for (i, &(addr, pc, val)) in log.iter().enumerate() {
        let raw_offset = (addr - 0x0600_0000) % 0x2_0000;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        if offset >= 394 * 32 && offset < 395 * 32 {
            tile_394_writes.push((i, pc, val));
        }
    }
    eprintln!("  {} total writes", tile_394_writes.len());
    for (i, pc, val) in tile_394_writes.iter() {
        eprintln!("    #{}: PC=0x{:08X} val=0x{:02X}", i, pc, val);
    }

    // Also check tile 0 (has data) for comparison
    eprintln!("\nFirst 20 writes to tile 0 (offset 0x0000-0x001F):");
    let mut count = 0;
    for (i, &(addr, pc, val)) in log.iter().enumerate() {
        let raw_offset = (addr - 0x0600_0000) % 0x2_0000;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        if offset < 32 {
            eprintln!("    #{}: PC=0x{:08X} val=0x{:02X}", i, pc, val);
            count += 1;
            if count >= 20 {
                break;
            }
        }
    }
}
