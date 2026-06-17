use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Enable palette write logging
    gba.mem.palette_log_enabled = true;

    // Run to title screen
    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    let pal_log = &gba.mem.palette_write_log;
    println!("Total palette writes: {}", pal_log.len());

    // Find writes to first 32 bytes (entries 0-15)
    let first_32: Vec<_> = pal_log
        .iter()
        .filter(|(addr, _)| *addr >= 0x05000000 && *addr < 0x05000020)
        .collect();
    println!(
        "Writes to PAL[0-15] (0x05000000-0x0500001F): {}",
        first_32.len()
    );

    for (addr, val) in first_32.iter().take(50) {
        let offset = (*addr - 0x05000000) as usize;
        let entry = offset / 2;
        println!(
            "  addr={:08X} offset={} entry={} val={:02X}",
            addr, offset, entry, val
        );
    }

    // Find writes to entries 16-31 (0x05000020-0x0500003F)
    let next_32: Vec<_> = pal_log
        .iter()
        .filter(|(addr, _)| *addr >= 0x05000020 && *addr < 0x05000040)
        .collect();
    println!(
        "\nWrites to PAL[16-31] (0x05000020-0x0500003F): {}",
        next_32.len()
    );
    for (addr, val) in next_32.iter().take(20) {
        let offset = (*addr - 0x05000000) as usize;
        let entry = offset / 2;
        println!("  addr={:08X} entry={} val={:02X}", addr, entry, val);
    }

    // Check current palette state
    let pal = gba.mem.palette();
    println!("\nCurrent palette[0-15]:");
    for i in 0..16 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        print!("  PAL[{}]={:04X}", i, c);
    }
    println!();

    // Check what addresses the writes target
    let mut addr_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for (addr, _) in pal_log.iter() {
        *addr_counts.entry(*addr & !3).or_insert(0) += 1;
    }
    println!("\nPalette write distribution (by 4-byte aligned address):");
    let mut addrs: Vec<_> = addr_counts.iter().collect();
    addrs.sort_by_key(|(a, _)| **a);
    for (addr, count) in addrs.iter().take(40) {
        let offset = (**addr - 0x05000000) as usize;
        let entry = offset / 2;
        println!(
            "  {:08X} (entry {}-{}): {} writes",
            addr,
            entry,
            entry + 1,
            count
        );
    }
}
