use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().dispcnt_write_log_enabled = true;

    for frame in 0..15 {
        let dma_count_before = gba.mem().dma_log.len();
        let dispcnt_writes_before = gba.mem().dispcnt_write_log.len();

        gba.run_frame_parallel(&mut framebuffer);

        let dma_count_after = gba.mem().dma_log.len();
        let dispcnt_writes_after = gba.mem().dispcnt_write_log.len();
        let dispcnt = gba.mem().io()[0] as u16 | ((gba.mem().io()[1] as u16) << 8);

        // New DMA3 IO transfers this frame
        let new_io_dmas: Vec<_> = gba.mem().dma_log[dma_count_before..dma_count_after]
            .iter()
            .filter(|&&(ch, _, dst, _, _)| ch == 3 && dst >= 0x04000000 && dst < 0x04000004)
            .collect();

        let new_dispcnt_writes = &gba.mem().dispcnt_write_log[dispcnt_writes_before..dispcnt_writes_after];

        println!(
            "Frame {}: DISPCNT={:#06X} (DMA_total={}, new_IO_DMA3={}, dispcnt_writes={})",
            frame,
            dispcnt,
            dma_count_after,
            new_io_dmas.len(),
            new_dispcnt_writes.len()
        );

        for &&(ch, src, dst, cnt, sz) in &new_io_dmas {
            // Read the first word from source at the time of this check
            let src_offset = (src - 0x02000000) as usize;
            let wram = gba.mem().wram();
            let buf_val = if src_offset + 4 <= wram.len() {
                u32::from_le_bytes([
                    wram[src_offset],
                    wram[src_offset + 1],
                    wram[src_offset + 2],
                    wram[src_offset + 3],
                ])
            } else {
                0xDEAD
            };
            println!(
                "  DMA3 IO copy: src={:#010X} cnt={} buf_DISPCNT={:#010X}",
                src, cnt, buf_val
            );
        }

        for &(pc, offset, val) in new_dispcnt_writes.iter() {
            println!(
                "  DISPCNT write: PC={:#010X} offset={} val={:#04X}",
                pc, offset, val
            );
        }
    }
}
