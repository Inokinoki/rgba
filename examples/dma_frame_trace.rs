use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.dma_log_enabled = true;

    for frame in 0..250u32 {
        let log_start = gba.mem.dma_log.len();

        gba.run_frame_parallel(&mut fb);

        // Check for DMA3 transfers to palette in this frame
        for (ch, src, dst, count, size) in &gba.mem.dma_log[log_start..] {
            if *ch == 3 && *dst >= 0x05000000 && *dst < 0x06000000 {
                println!(
                    "Frame {}: DMA3 src={:08X} dst={:08X} count={} size={}",
                    frame, src, dst, count, size
                );
            }
        }

        // Check palette after each frame
        let pal = gba.mem.palette();
        let pal0 = u16::from_le_bytes([pal[0], pal[1]]);
        if frame >= 188 && frame <= 196 {
            println!("  -> PAL[0]={:04X}", pal0);
        }
    }
}
