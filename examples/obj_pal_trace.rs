use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.palette_log_enabled = true;

    for frame in 0..500u32 {
        let log_start = gba.mem.palette_write_log.len();

        gba.run_frame_parallel(&mut fb);

        // Find writes to OBJ palette (0x05000200-0x050003FF)
        let obj_writes: Vec<_> = gba.mem.palette_write_log[log_start..]
            .iter()
            .filter(|(addr, _, _, _)| *addr >= 0x05000200 && *addr < 0x05000400)
            .collect();

        if !obj_writes.is_empty() {
            println!("Frame {}: {} OBJ palette writes", frame, obj_writes.len());
            for (addr, pc, val, dma) in obj_writes.iter().take(20) {
                println!(
                    "  addr={:08X} val={:02X} pc={:08X} dma={}",
                    addr, val, pc, dma
                );
            }
        }
    }

    // Summary
    let all_obj_writes: Vec<_> = gba
        .mem
        .palette_write_log
        .iter()
        .filter(|(addr, _, _, _)| *addr >= 0x05000200 && *addr < 0x05000400)
        .collect();
    println!("\nTotal OBJ palette writes: {}", all_obj_writes.len());

    // Also check BG palette writes for comparison
    let bg_writes: Vec<_> = gba
        .mem
        .palette_write_log
        .iter()
        .filter(|(addr, _, _, _)| *addr >= 0x05000000 && *addr < 0x05000200)
        .collect();
    println!("Total BG palette writes: {}", bg_writes.len());

    // Check DMA transfers targeting 0x05000200 region
    println!("\n--- DMA log for palette range ---");
    gba.mem.dma_log_enabled = true;
    // Run a few more frames
    for frame in 500..510u32 {
        let log_start = gba.mem.dma_log.len();
        gba.run_frame_parallel(&mut fb);
        for (ch, src, dst, count, size) in &gba.mem.dma_log[log_start..] {
            if *dst >= 0x05000000 && *dst < 0x06000000 {
                println!(
                    "Frame {}: DMA{} src={:08X} dst={:08X} count={} size={}",
                    frame, ch, src, dst, count, size
                );
            }
        }
    }
}
