use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    // Run to frame 192 where palette gets zeroed
    for _ in 0..193 {
        gba.run_frame_parallel(&mut fb);
    }

    // Find all writes to 0x05000000-0x0500001F that have value 0
    let pal_log = &gba.mem.palette_write_log;
    let zero_writes: Vec<_> = pal_log
        .iter()
        .filter(|(addr, pc, val)| {
            let offset = if *addr >= 0x05000000 && *addr < 0x05010000 {
                (*addr - 0x05000000) & 0x3FF
            } else {
                0xFFFF
            };
            offset < 0x20 && *val == 0
        })
        .collect();

    println!("Zero writes to PAL[0-15] range:");
    for (addr, pc, val) in zero_writes.iter().take(40) {
        let offset = (*addr - 0x05000000) & 0x3FF;
        let entry = offset / 2;
        println!(
            "  addr={:08X} pc={:08X} val={:02X} (entry {})",
            addr, pc, val, entry
        );
    }

    // Also check ALL writes in the last batch (near frame 192)
    // The writes are in chronological order, so let's look at the last group
    println!("\nLast 60 palette writes:");
    let start = pal_log.len().saturating_sub(60);
    for (addr, pc, val) in pal_log[start..].iter() {
        let offset = (*addr - 0x05000000) & 0x3FF;
        let entry = offset / 2;
        let is_mirror = *addr >= 0x05000400;
        println!(
            "  addr={:08X}{} pc={:08X} val={:02X} (entry {})",
            addr,
            if is_mirror { " [MIRROR]" } else { "" },
            pc,
            val,
            entry
        );
    }
}
