use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.vram_log_enabled = true;

    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }

    // Find writes to VRAM 0xC000-0xCFFF (BG0 screen entries)
    let log = &gba.mem.vram_write_log;
    eprintln!("Total VRAM writes: {}", log.len());

    let mut screen_entry_writes: Vec<(u32, u32, u8)> = Vec::new();
    for &(addr, pc, val) in log {
        let raw_offset = ((addr - 0x0600_0000) % 0x2_0000) as usize;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        // Check if write is to screen entry area 0xC000-0xCFFF
        if offset >= 0xC000 && offset < 0xD000 {
            screen_entry_writes.push((addr, pc, val));
        }
    }

    eprintln!(
        "Writes to BG0 screen entry area (0xC000-0xCFFF): {}",
        screen_entry_writes.len()
    );

    // Show first 50 writes
    eprintln!("\nFirst 50 writes:");
    for (i, (addr, pc, val)) in screen_entry_writes.iter().take(50).enumerate() {
        eprintln!(
            "  #{}: addr=0x{:08X} pc=0x{:08X} val=0x{:02X}",
            i, addr, pc, val
        );
    }

    // What PCs write to screen entries?
    let mut pc_counts: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for &(_, pc, _) in &screen_entry_writes {
        *pc_counts.entry(pc).or_insert(0) += 1;
    }
    eprintln!("\nPCs writing to screen entries:");
    for (pc, count) in &pc_counts {
        eprintln!("  PC=0x{:08X}: {} writes", pc, count);
    }

    // Check DMA to screen entries
    gba.mem.dma_log_enabled = true;
    for &(num, src, dst, count, size) in &gba.mem.dma_log {
        let raw_offset = ((dst - 0x0600_0000) % 0x2_0000) as usize;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        if offset >= 0xC000 && offset < 0xD000 {
            eprintln!(
                "\nDMA to screen entry area: DMA{} src=0x{:08X} dst=0x{:08X} count={} size={}",
                num, src, dst, count, size
            );
        }
    }
}
