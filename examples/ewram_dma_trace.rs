use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.decomp_writes_enabled = true;
    gba.mem.dma_log_enabled = true;
    gba.mem.cpu_set_log_enabled = true;

    for frame in 0..300u32 {
        let dw_before = gba.mem.decomp_writes.len();
        let dma_before = gba.mem.dma_log.len();
        let cpuset_before = gba.mem.cpu_set_log.len();

        gba.run_frame_parallel(&mut fb);

        let new_dw = gba.mem.decomp_writes.len() - dw_before;
        let new_dma = gba.mem.dma_log.len() - dma_before;
        let new_cpuset = gba.mem.cpu_set_log.len() - cpuset_before;

        if new_dma > 0 || new_cpuset > 0 || new_dw > 0 {
            let dma_to_8k: Vec<_> = gba.mem.dma_log[dma_before..]
                .iter()
                .filter(|(_, dst, _, _, _)| *dst >= 0x02008000 && *dst < 0x02009000)
                .collect();
            let cpuset_to_8k: Vec<_> = gba.mem.cpu_set_log[cpuset_before..]
                .iter()
                .filter(|(_, dst, _)| *dst >= 0x02008000 && *dst < 0x02009000)
                .collect();
            if !dma_to_8k.is_empty() || !cpuset_to_8k.is_empty() || new_dw > 0 {
                println!(
                    "Frame {:4}: dw={} dma_to_8k={} cpuset_to_8k={}",
                    frame,
                    new_dw,
                    dma_to_8k.len(),
                    cpuset_to_8k.len()
                );
                for (ch, dst, src, cnt, ctrl) in dma_to_8k.iter() {
                    println!(
                        "  DMA{} {:08X}->{:08X} len={} ctrl={:08X}",
                        ch, src, dst, cnt, ctrl
                    );
                }
                for (src, dst, cnt) in cpuset_to_8k.iter() {
                    println!("  CpuSet {:08X}->{:08X} cnt={}", src, dst, cnt);
                }
            }
        }
    }

    // Also check: what DMA/CpuSet transfers write to EWRAM 0x2000000-0x2004000?
    println!("\n=== All DMA to EWRAM (first 50) ===");
    let mut count = 0;
    for (ch, dst, src, cnt, ctrl) in gba.mem.dma_log.iter() {
        if *dst >= 0x02000000 && *dst < 0x02040000 {
            println!(
                "  DMA{} {:08X}->{:08X} len={} ctrl={:08X}",
                ch, src, dst, cnt, ctrl
            );
            count += 1;
            if count >= 50 {
                break;
            }
        }
    }

    println!("\n=== All CpuSet to EWRAM (first 50) ===");
    count = 0;
    for (src, dst, cnt) in gba.mem.cpu_set_log.iter() {
        if *dst >= 0x02000000 && *dst < 0x02040000 {
            println!("  CpuSet {:08X}->{:08X} cnt={}", src, dst, cnt);
            count += 1;
            if count >= 50 {
                break;
            }
        }
    }
}
