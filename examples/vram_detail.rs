use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for frame in 0..7 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    let log = &gba.mem().vram_write_log;
    println!("Total writes after 7 frames: {}", log.len());

    // Show address range of writes
    let mut min_addr = u32::MAX;
    let mut max_addr = 0u32;
    for &(addr, _pc, _val) in log.iter() {
        min_addr = min_addr.min(addr);
        max_addr = max_addr.max(addr);
    }
    println!("Address range: {:08X} - {:08X}", min_addr, max_addr);

    // Histogram by 1KB chunks
    let mut buckets = vec![0usize; 128]; // 128KB / 1KB = 128 buckets
    for &(addr, _pc, _val) in log.iter() {
        let offset = (addr - 0x06000000) as usize;
        let bucket = offset / 1024;
        if bucket < 128 {
            buckets[bucket] += 1;
        }
    }
    println!("\n=== VRAM write histogram (per 1KB) ===");
    for (i, &count) in buckets.iter().enumerate() {
        if count > 0 {
            println!(
                "  0x{:05X}-0x{:05X}: {} writes",
                i * 1024,
                (i + 1) * 1024 - 1,
                count
            );
        }
    }

    // Check BG tile area specifically (0x00000-0x0EFFF)
    let bg_tile_writes: Vec<_> = log
        .iter()
        .filter(|(addr, _, _)| {
            let off = addr - 0x06000000;
            off < 0x0F000
        })
        .collect();
    println!(
        "\nBG tile area writes (0x00000-0x0EFFF): {}",
        bg_tile_writes.len()
    );

    if !bg_tile_writes.is_empty() {
        for (addr, pc, val) in bg_tile_writes.iter().take(20) {
            println!("  addr={:08X} pc={:08X} val={:02X}", addr, pc, val);
        }
    }
}
