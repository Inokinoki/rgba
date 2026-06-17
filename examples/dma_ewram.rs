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

        // Check ALL DMA transfers that write to EWRAM anywhere
        for (ch, src, dst, count, size) in &gba.mem.dma_log[log_start..] {
            // DMA to EWRAM
            if *dst >= 0x02000000 && *dst < 0x03000000 && *ch == 3 {
                let end = dst + count * (*size as u32);
                if *dst >= 0x02008000 && *dst < 0x0200A000 {
                    println!(
                        "Frame {}: DMA3 EWRAM src={:08X} dst={:08X}-{:08X} count={} size={}B",
                        frame, src, dst, end, count, size
                    );
                }
            }
        }
    }
}
