use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.dma_log_enabled = true;
    gba.mem.ewram_tile_log_enabled = true;

    for frame in 0..192u32 {
        let dma_start = gba.mem.dma_log.len();
        let ewram_start = gba.mem.ewram_tile_log.len();

        gba.run_frame_parallel(&mut fb);

        // Check for DMA transfers that write to the problem EWRAM region
        for (ch, src, dst, count, size) in &gba.mem.dma_log[dma_start..] {
            let end_addr = dst + count * (*size as u32);
            // Check if this DMA writes to 0x02008000-0x02009000
            if *dst >= 0x02008000 && *dst < 0x02009000 && *ch == 3 {
                println!(
                    "Frame {}: DMA{} src={:08X} dst={:08X}-{:08X} count={} size={}B",
                    frame, ch, src, dst, end_addr, count, size
                );
            }
        }

        // Check for EWRAM writes to the problem region
        let ewram_writes: Vec<_> = gba.mem.ewram_tile_log[ewram_start..]
            .iter()
            .filter(|(addr, _, _)| *addr >= 0x02008000 && *addr < 0x02009000)
            .collect();
        if !ewram_writes.is_empty() {
            println!(
                "Frame {}: {} EWRAM writes to 0x2008000-0x2009000",
                frame,
                ewram_writes.len()
            );
            for (addr, pc, val) in ewram_writes.iter().take(10) {
                println!("  addr={:08X} val={:02X} pc={:08X}", addr, val, pc);
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total DMA log entries: {}", gba.mem.dma_log.len());

    let ewram_to_region: Vec<_> = gba
        .mem
        .ewram_tile_log
        .iter()
        .filter(|(addr, _, _)| *addr >= 0x02008000 && *addr < 0x02009000)
        .collect();
    println!(
        "Total EWRAM writes to 0x2008000-0x2009000: {}",
        ewram_to_region.len()
    );
}
