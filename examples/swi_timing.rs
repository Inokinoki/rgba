use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Track per-frame: SWI 0x01 calls, DMA3 to palette, and PAL[0] changes
    for frame in 0..250u32 {
        let pal_before = {
            let pal = gba.mem.palette();
            u16::from_le_bytes([pal[0], pal[1]])
        };

        // Clear SWI log for this frame
        gba.mem.swi_log.clear();
        gba.mem.dma_log.clear();

        gba.run_frame_parallel(&mut fb);

        let pal_after = {
            let pal = gba.mem.palette();
            u16::from_le_bytes([pal[0], pal[1]])
        };

        // Check for SWI 0x01 in this frame
        let swi01_count = gba.mem.swi_log.iter().filter(|&&s| s == 0x01).count();

        // Check for DMA3 to palette
        let dma_pal: Vec<_> = gba
            .mem
            .dma_log
            .iter()
            .filter(|(ch, _, dst, _, _)| *ch == 3 && *dst >= 0x05000000 && *dst < 0x06000000)
            .collect();

        // Check for RegisterRamReset with palette flag
        // SWI 0x01 uses R0 as flags; we need to check R0 at the time of call
        // But we can't easily get R0 from the log. Let's just report the events.

        if pal_before != pal_after || swi01_count > 0 || !dma_pal.is_empty() {
            print!(
                "Frame {}: PAL[0] {:04X}->{:04X}",
                frame, pal_before, pal_after
            );
            if swi01_count > 0 {
                print!(" SWI0x01(x{})", swi01_count);
            }
            for (ch, src, dst, count, size) in dma_pal {
                print!(" DMA{}({:X}->{:X},{},{})", ch, src, dst, count, size);
            }
            println!();
        }
    }
}
