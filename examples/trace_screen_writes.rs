use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    gba.mem.vram_log_enabled = true;
    gba.mem.dma_log_enabled = true;

    for _frame in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.mem.vram_log_enabled = false;
    gba.mem.dma_log_enabled = false;

    println!(
        "Total VRAM write log entries: {}",
        gba.mem.vram_write_log.len()
    );
    println!("Total DMA log entries: {}", gba.mem.dma_log.len());

    let vram = gba.mem.vram();
    println!("\n=== Current screen entry values at VRAM+0xC000 (first 64 entries) ===");
    for i in 0..64u32 {
        let offset = 0xC000 + (i as usize) * 2;
        let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
        let tile = entry & 0x3FF;
        let pal = (entry >> 12) & 0xF;
        print!(" [{:3}]={:04X}(t={:4}p={:2})", i, entry, tile, pal);
        if (i + 1) % 4 == 0 {
            println!();
        }
    }

    println!("\n=== Screen entries: border vs interior ===");
    let mut border_ok = 0usize;
    let mut border_bad = 0usize;
    let mut interior_ok = 0usize;
    let mut interior_bad = 0usize;
    for row in 0..32u32 {
        for col in 0..32u32 {
            let is_border = row == 0 || row == 31 || col == 0 || col == 31;
            let offset = 0xC000 + ((row * 32 + col) as usize) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            let pal = (entry >> 12) & 0xF;
            if pal == 11 {
                if is_border {
                    border_ok += 1;
                } else {
                    interior_ok += 1;
                }
            } else {
                if is_border {
                    border_bad += 1;
                } else {
                    interior_bad += 1;
                }
            }
        }
    }
    println!("  Border:   {} pal=11, {} pal!=11", border_ok, border_bad);
    println!(
        "  Interior: {} pal=11, {} pal!=11",
        interior_ok, interior_bad
    );

    let screen_base: u32 = 0x0600C000;
    let screen_end: u32 = 0x0600C800;
    let screen_writes: Vec<_> = gba
        .mem
        .vram_write_log
        .iter()
        .filter(|(addr, _, _)| *addr >= screen_base && *addr < screen_end)
        .collect();
    println!(
        "\n=== VRAM byte-writes to 0x0600C000-0x0600C7FF: {} ===",
        screen_writes.len()
    );

    if !screen_writes.is_empty() {
        let mut pc_counts: std::collections::BTreeMap<u32, usize> =
            std::collections::BTreeMap::new();
        for &&(_addr, pc, _val) in &screen_writes {
            *pc_counts.entry(pc).or_insert(0) += 1;
        }
        println!("PCs writing to screen entries:");
        for (pc, count) in pc_counts.iter().rev() {
            let src = if *pc >= 0x08000000 {
                "CPU-ROM"
            } else if *pc >= 0x03000000 {
                "CPU-IWRAM"
            } else if *pc >= 0x02000000 {
                "CPU-EWRAM"
            } else {
                "DMA/stale"
            };
            println!("  {:5} writes PC=0x{:08X} ({})", count, pc, src);
        }

        println!("\nFirst 40 writes:");
        for (i, &&(addr, pc, val)) in screen_writes.iter().take(40).enumerate() {
            let byte_role = if (addr & 1) == 0 { "lo" } else { "hi" };
            println!(
                "  {:3}: 0x{:08X}({}) val=0x{:02X} pc=0x{:08X}",
                i, addr, byte_role, val, pc
            );
        }
    } else {
        println!("  NO writes logged! Screen entries are written through unlogged path.");
        println!("  This confirms the write_byte() VRAM path bypasses vram_write_log.");
    }

    println!("\n=== DMA transfers ({} total) ===", gba.mem.dma_log.len());
    let mut dma_to_screen = 0;
    for &(ch, src, dst, count, size) in &gba.mem.dma_log {
        let src_region = if src >= 0x08000000 {
            "ROM"
        } else if src >= 0x02000000 {
            "EWRAM"
        } else if src >= 0x03000000 {
            "IWRAM"
        } else {
            "???"
        };
        let dst_off = if dst >= 0x06000000 {
            (dst - 0x06000000) % 0x20000
        } else {
            0xFFFFFFFF
        };
        let covers_screen = dst_off < 0xC800 && dst_off != 0xFFFFFFFF;
        if covers_screen {
            dma_to_screen += 1;
        }
        if dst_off < 0x18000 {
            let dst_desc = if dst_off < 0x10000 { "BG" } else { "OBJ" };
            if covers_screen {
                println!(
                    "  >>> DMA{}: {}(0x{:08X}) -> {}+0x{:05X} count=0x{:X} size={} <<<",
                    ch, src_region, src, dst_desc, dst_off, count, size
                );
            }
        }
    }
    println!(
        "  DMA transfers covering screen entry area: {}",
        dma_to_screen
    );

    println!("\n=== Writes to VRAM by region ===");
    let mut tile_writes = 0usize;
    let mut map_writes = 0usize;
    let mut obj_writes = 0usize;
    let mut other_vram = 0usize;
    for (addr, _, _) in &gba.mem.vram_write_log {
        if *addr >= 0x06000000 && *addr < 0x0600C000 {
            tile_writes += 1;
        } else if *addr >= 0x0600C000 && *addr < 0x06010000 {
            map_writes += 1;
        } else if *addr >= 0x06010000 && *addr < 0x06018000 {
            obj_writes += 1;
        } else if *addr >= 0x06000000 {
            other_vram += 1;
        }
    }
    println!("  Tile (0x06000000-0x0600BFFF): {}", tile_writes);
    println!("  Map  (0x0600C000-0x0600FFFF): {}", map_writes);
    println!("  OBJ  (0x06010000-0x06017FFF): {}", obj_writes);
    println!("  Other VRAM:                     {}", other_vram);

    let non_zero_map: Vec<_> = gba
        .mem
        .vram_write_log
        .iter()
        .filter(|(a, _, _)| *a >= 0x0600C000 && *a < 0x06010000)
        .take(20)
        .collect();
    if !non_zero_map.is_empty() {
        println!("\n  Sample map writes:");
        for &(addr, pc, val) in &non_zero_map {
            println!("    addr=0x{:08X} val=0x{:02X} pc=0x{:08X}", addr, val, pc);
        }
    }
}
