use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;

    for frame in 0..30 {
        gba.run_frame_parallel(&mut framebuffer);

        let dispcnt = gba.mem().io()[0] as u16 | ((gba.mem().io()[1] as u16) << 8);
        println!("Frame {}: DISPCNT={:#06X}", frame, dispcnt);
    }

    let dma_log = &gba.mem().dma_log;

    println!("\n=== All DMA transfers ===");
    println!("Total DMA transfers: {}", dma_log.len());

    // Show all DMA3 transfers to IO space
    let io_dmas: Vec<_> = dma_log
        .iter()
        .filter(|&&(ch, src, dst, cnt, sz)| {
            ch == 3 && dst >= 0x04000000 && dst < 0x04000004
        })
        .collect();

    println!("\nDMA3 transfers to IO (DISPCNT area): {}", io_dmas.len());
    for (idx, &&(ch, src, dst, cnt, sz)) in io_dmas.iter().enumerate() {
        println!(
            "  [{}] DMA{} src={:#010X} dst={:#010X} cnt={} sz={}",
            idx, ch, src, dst, cnt, sz
        );

        // Read the first word from source (EWRAM)
        let src_offset = (src - 0x02000000) as usize;
        let wram = gba.mem().wram();
        if src_offset + 4 <= wram.len() {
            let val = u32::from_le_bytes([
                wram[src_offset],
                wram[src_offset + 1],
                wram[src_offset + 2],
                wram[src_offset + 3],
            ]);
            println!(
                "    First word at src: {:#010X} (DISPCNT would be {:#06X})",
                val,
                val as u16
            );
        }
    }

    // Also show ALL DMA3 transfers
    let dma3_all: Vec<_> = dma_log.iter().filter(|&&(ch, _, _, _, _)| ch == 3).collect();
    println!("\nAll DMA3 transfers: {}", dma3_all.len());
    for (idx, &&(ch, src, dst, cnt, sz)) in dma3_all.iter().enumerate().take(20) {
        println!(
            "  [{}] DMA{} src={:#010X} dst={:#010X} cnt={} sz={}",
            idx, ch, src, dst, cnt, sz
        );
    }

    // Check EWRAM buffer at 0x02008D2C
    let buf_offset = (0x02008D2C - 0x02000000) as usize;
    let wram = gba.mem().wram();
    println!("\n=== EWRAM IO buffer at 0x02008D2C (after 30 frames) ===");
    for i in 0..21 {
        let off = buf_offset + i * 4;
        if off + 4 <= wram.len() {
            let val = u32::from_le_bytes([wram[off], wram[off + 1], wram[off + 2], wram[off + 3]]);
            let io_addr = 0x04000000 + (i * 4) as u32;
            println!("  [{:2}] {:#010X} -> IO {:#010X}: {:#010X}", i, 0x02008D2C + (i*4) as u32, io_addr, val);
        }
    }

    // Also check what DISPCNT is in the buffer vs actual
    if buf_offset + 4 <= wram.len() {
        let buf_dispcnt = u16::from_le_bytes([wram[buf_offset], wram[buf_offset + 1]]);
        let actual_dispcnt = gba.mem().io()[0] as u16 | ((gba.mem().io()[1] as u16) << 8);
        println!(
            "\nBuffer DISPCNT={:#06X}, Actual DISPCNT={:#06X}",
            buf_dispcnt, actual_dispcnt
        );
    }
}
