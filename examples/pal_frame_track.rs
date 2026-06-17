use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Enable DMA logging to see DMA transfers targeting palette area
    gba.mem.dma_log_enabled = true;
    gba.mem.palette_log_enabled = true;

    // Run frame by frame, checking palette state
    for frame in 0..500u32 {
        // Check palette before frame
        let pal = gba.mem.palette();
        let pal0 = u16::from_le_bytes([pal[0], pal[1]]);
        let pal1 = u16::from_le_bytes([pal[2], pal[3]]);

        gba.run_frame_parallel(&mut fb);

        // Check palette after frame
        let pal = gba.mem.palette();
        let pal0_after = u16::from_le_bytes([pal[0], pal[1]]);
        let pal1_after = u16::from_le_bytes([pal[2], pal[3]]);

        if pal0 != pal0_after || pal1 != pal1_after {
            println!(
                "Frame {}: PAL[0] {:04X}->{:04X}, PAL[1] {:04X}->{:04X}",
                frame, pal0, pal0_after, pal1, pal1_after
            );
        }
    }

    // Find DMA transfers targeting palette area (0x05000000-0x05FFFFFF)
    let dma_log = &gba.mem.dma_log;
    println!("\nDMA transfers targeting palette (0x05000000+):");
    for (ch, src, dst, count, size) in dma_log.iter() {
        if *dst >= 0x05000000 && *dst < 0x06000000 {
            println!(
                "  DMA{} src={:08X} dst={:08X} count={} size={}",
                ch, src, dst, count, size
            );
        }
    }
    println!("Total DMA transfers: {}", dma_log.len());
}
