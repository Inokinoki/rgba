use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Check source data for the DMA3 count=8 transfer
    // src=0x084D529C in ROM
    let rom = gba.mem.rom();
    let src1_offset = 0x084D529C - 0x08000000;
    let src2_offset = 0x084D4AC4 - 0x08000000;

    println!(
        "DMA3 count=8 source at 0x084D529C (offset {:X}):",
        src1_offset
    );
    if src1_offset + 32 <= rom.len() {
        for i in 0..8 {
            let w = u32::from_le_bytes([
                rom[src1_offset + i * 4],
                rom[src1_offset + i * 4 + 1],
                rom[src1_offset + i * 4 + 2],
                rom[src1_offset + i * 4 + 3],
            ]);
            print!("{:08X} ", w);
        }
        println!();
    }

    println!(
        "\nDMA3 count=128 source at 0x084D4AC4 (offset {:X}):",
        src2_offset
    );
    if src2_offset + 32 <= rom.len() {
        for i in 0..16 {
            let w = u32::from_le_bytes([
                rom[src2_offset + i * 4],
                rom[src2_offset + i * 4 + 1],
                rom[src2_offset + i * 4 + 2],
                rom[src2_offset + i * 4 + 3],
            ]);
            print!("{:08X} ", w);
        }
        println!();
    }

    // Run with detailed DMA logging per frame
    gba.mem.dma_log_enabled = true;

    // Read ROM data before enabling log
    let rom = gba.mem.rom().to_vec();

    for frame in 0..500u32 {
        let log_start = gba.mem.dma_log.len();
        gba.run_frame_parallel(&mut fb);

        // Check for DMA3 transfers to palette in this frame
        for (ch, src, dst, count, size) in &gba.mem.dma_log[log_start..] {
            if *ch == 3 && *dst >= 0x05000000 && *dst < 0x06000000 {
                let src_off = (*src - 0x08000000) as usize;
                let first_word = if src_off + 4 <= rom.len() {
                    u32::from_le_bytes([
                        rom[src_off],
                        rom[src_off + 1],
                        rom[src_off + 2],
                        rom[src_off + 3],
                    ])
                } else {
                    0
                };
                println!(
                    "Frame {}: DMA3 src={:08X} dst={:08X} count={} size={} first_word={:08X}",
                    frame, src, dst, count, size, first_word
                );
            }
        }

        // Check palette state
        let pal = gba.mem.palette();
        let pal0 = u16::from_le_bytes([pal[0], pal[1]]);
        if frame >= 189 && frame <= 195 {
            println!(
                "  Frame {} end: PAL[0]={:04X} PAL[1]={:04X} PAL[16]={:04X}",
                frame,
                pal0,
                u16::from_le_bytes([pal[2], pal[3]]),
                u16::from_le_bytes([pal[32], pal[33]])
            );
        }
    }
}
