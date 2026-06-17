use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.dma_log_enabled = true;

    for frame in 0..200u32 {
        let log_start = gba.mem.dma_log.len();

        gba.run_frame_parallel(&mut fb);

        if frame == 192 {
            println!("=== Frame 192 DMA transfers ===");
            let all = &gba.mem.dma_log[log_start..];
            if all.is_empty() {
                println!("  (no DMA transfers)");
            }
            for (ch, src, dst, count, size) in all {
                println!(
                    "  DMA{}: src={:08X} dst={:08X} count={} size={}B",
                    ch, src, dst, count, size
                );
            }
        }
    }
}
