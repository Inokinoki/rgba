use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.dma_log_enabled = true;

    for frame in 0..10u32 {
        let log_start = gba.mem.dma_log.len();

        gba.run_frame_parallel(&mut fb);

        for (ch, src, dst, count, size) in &gba.mem.dma_log[log_start..] {
            let end = dst + count * (*size as u32);
            // Check ALL DMA transfers to EWRAM 0x02008000-0x02010000
            if *dst >= 0x02008000 && *dst < 0x02010000 {
                println!(
                    "Frame {}: DMA{} src={:08X} dst={:08X}-{:08X} count={} size={}B",
                    frame, ch, src, dst, end, count, size
                );
            }
            // Also check any DMA that reads from 0x02008000+
            if *src >= 0x02008000 && *src < 0x02010000 && !(*dst >= 0x02008000 && *dst < 0x02010000)
            {
                println!(
                    "Frame {}: DMA{} src={:08X} dst={:08X}-{:08X} count={} size={}B (READS from EWRAM)",
                    frame, ch, src, dst, end, count, size
                );
            }
        }
    }
}
