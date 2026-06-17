use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().dma_log.clear();
    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().swi_log.clear();
    gba.mem_mut().cpu_set_log_enabled = true;
    gba.mem_mut().cpu_set_log.clear();
    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    let mut prev_dma_len = 0;
    let mut prev_swi_len = 0;
    let mut prev_cpuset_len = 0;
    let mut prev_vram_len = 0;

    for frame in 0..300 {
        for _ in 0..228 {
            gba.run_scanline();
        }

        let dma_log = &gba.mem().dma_log;
        let swi_log = &gba.mem().swi_log;
        let cpu_set = &gba.mem().cpu_set_log;
        let vram_log = &gba.mem().vram_write_log;

        let new_dma = dma_log.len() - prev_dma_len;
        let new_swi = swi_log.len() - prev_swi_len;
        let new_cpuset = cpu_set.len() - prev_cpuset_len;
        let new_vram = vram_log.len() - prev_vram_len;

        if new_dma > 0 || new_swi > 0 || new_cpuset > 0 || new_vram > 0 {
            let mut new_dma_entries: Vec<_> = dma_log[prev_dma_len..]
                .iter()
                .filter(|&&(_, _, dst, _, _)| dst >= 0x06000000 && dst < 0x06018000)
                .collect();
            let vram_dma = new_dma_entries.len();

            // Count BG VRAM writes (not screen blocks)
            let bg_tile_writes: usize = vram_log[prev_vram_len..]
                .iter()
                .filter(|(addr, _, _)| {
                    let off = *addr - 0x06000000;
                    off < 0x0F000 // BG tile area, not screen blocks
                })
                .count();

            println!(
                "Frame {:3}: +{} DMA ({} VRAM), +{} SWI, +{} CpuSet, +{} VRAM writes ({} BG tiles)",
                frame, new_dma, vram_dma, new_swi, new_cpuset, new_vram, bg_tile_writes
            );

            // Show new VRAM-targeted DMA
            for &(ch, src, dst, cnt, ctrl) in &dma_log[prev_dma_len..] {
                if dst >= 0x06000000 && dst < 0x06018000 {
                    let area = if dst >= 0x06010000 {
                        "OBJ"
                    } else if dst >= 0x0600F000 {
                        "SCR"
                    } else {
                        "BG"
                    };
                    println!(
                        "    DMA{}: {:08X} -> {:08X} ({}w) [{}]",
                        ch, src, dst, cnt, area
                    );
                }
            }

            // Show new CpuSet that targets VRAM
            for &(src, dst, cnt) in &cpu_set[prev_cpuset_len..] {
                if dst >= 0x06000000 && dst < 0x06018000 {
                    println!(
                        "    CpuSet: {:08X} -> {:08X} ({}w)",
                        src,
                        dst,
                        cnt & 0xFFFFFF
                    );
                }
            }
        }

        prev_dma_len = dma_log.len();
        prev_swi_len = swi_log.len();
        prev_cpuset_len = cpu_set.len();
        prev_vram_len = vram_log.len();
    }
}
