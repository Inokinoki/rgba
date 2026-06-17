use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    for _ in 0..193 {
        gba.run_frame_parallel(&mut fb);
    }

    // Find zero writes to PAL[0-15] range and check if DMA
    let pal_log = &gba.mem.palette_write_log;
    println!("Zero writes to palette mirror (0x05000400+) or entries 0-15:");
    for (addr, pc, val, is_dma) in pal_log.iter() {
        let offset = if *addr >= 0x05000000 && *addr < 0x05010000 {
            (*addr - 0x05000000) & 0x3FF
        } else {
            0xFFFF
        };
        if *val == 0 && (offset < 0x20 || *addr >= 0x05000400) {
            let entry = offset / 2;
            println!(
                "  addr={:08X} pc={:08X} dma={} entry={}",
                addr, pc, is_dma, entry
            );
        }
    }

    // Also show last 20 writes regardless
    println!("\nLast 20 palette writes (all):");
    let start = pal_log.len().saturating_sub(20);
    for (addr, pc, val, is_dma) in pal_log[start..].iter() {
        let offset = (*addr - 0x05000000) & 0x3FF;
        println!(
            "  addr={:08X} pc={:08X} val={:02X} dma={} entry={}",
            addr,
            pc,
            val,
            is_dma,
            offset / 2
        );
    }

    // Count DMA vs non-DMA writes
    let (dma_count, cpu_count) = pal_log
        .iter()
        .filter(|(addr, _, val, _)| {
            let offset = (*addr - 0x05000000) & 0x3FF;
            *val == 0 && offset < 0x20
        })
        .fold(
            (0, 0),
            |(d, c), (_, _, _, is_dma)| {
                if *is_dma {
                    (d + 1, c)
                } else {
                    (d, c + 1)
                }
            },
        );
    println!(
        "\nZero writes to entries 0-15: {} DMA, {} CPU",
        dma_count, cpu_count
    );
}
