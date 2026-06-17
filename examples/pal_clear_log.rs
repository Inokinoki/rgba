use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;
    gba.mem.swi_log_enabled = true;

    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check SWI calls
    let swi_log = &gba.mem.swi_log;
    println!("SWI calls: {} total", swi_log.len());
    let mut swi_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &swi in swi_log.iter() {
        *swi_counts.entry(swi).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = swi_counts.iter().collect();
    sorted.sort_by_key(|(_, &c)| std::cmp::Reverse(c));
    for (&swi, &count) in sorted {
        println!("  SWI 0x{:02X}: {} calls", swi, count);
    }

    // Find where palette is cleared
    // Look at the last write to PAL[0] and check if SWI 0x01 with bit 2 comes after
    let pal_log = &gba.mem.palette_write_log;
    let first_writes: Vec<_> = pal_log
        .iter()
        .filter(|(addr, _)| *addr >= 0x05000000 && *addr < 0x05000020)
        .collect();

    // The second pass starts at index 32 (first 32 bytes = 32 entries)
    println!("\nSecond pass writes to PAL[0-15]:");
    if first_writes.len() >= 32 {
        for (addr, val) in first_writes[32..].iter().take(32) {
            let offset = (*addr - 0x05000000) as usize;
            let entry = offset / 2;
            let word = u16::from_le_bytes([*val, 0]); // approximate
            println!("  PAL[{}] <= {:02X}", entry, val);
        }
    }

    // Check: was there a SWI 0x01 call?
    let ram_reset_count = swi_counts.get(&0x01).unwrap_or(&0);
    println!(
        "\nSWI 0x01 (RegisterRamReset) called {} times",
        ram_reset_count
    );

    // Now check: was palette cleared AFTER the writes?
    // The total log has interleaved SWI and palette writes
    // Let me check the palette write indices to see when writes happen
    println!("\nLast 10 palette writes:");
    for (addr, val) in pal_log.iter().rev().take(10) {
        let offset = (*addr - 0x05000000) as usize;
        println!("  addr={:08X} val={:02X} (entry {})", addr, val, offset / 2);
    }

    // Check current palette
    let pal = gba.mem.palette();
    println!(
        "\nCurrent PAL[0]={:02X}{:02X}, PAL[1]={:02X}{:02X}",
        pal[1], pal[0], pal[3], pal[2]
    );
    println!(
        "Current PAL[16]={:02X}{:02X}, PAL[17]={:02X}{:02X}",
        pal[33], pal[32], pal[35], pal[34]
    );
}
