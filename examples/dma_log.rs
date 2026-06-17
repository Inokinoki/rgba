use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().dma_log.clear();

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let dma_log = &gba.mem().dma_log;
    println!("DMA transfers logged: {}", dma_log.len());

    let mut vram_writes = 0;
    let mut rom_reads = 0;
    for (ch, src, dst, count, size) in dma_log {
        if *dst >= 0x0600_0000 && *dst < 0x0602_0000 {
            vram_writes += 1;
        }
        if *src >= 0x0800_0000 {
            rom_reads += 1;
        }
    }
    println!("DMA transfers to VRAM: {}", vram_writes);
    println!("DMA transfers from ROM: {}", rom_reads);

    println!("\nFirst 20 DMA transfers:");
    for (i, (ch, src, dst, count, size)) in dma_log.iter().take(20).enumerate() {
        let is_vram = *dst >= 0x0600_0000 && *dst < 0x0602_0000;
        let is_rom = *src >= 0x0800_0000;
        println!(
            "  DMA{}: src={:#010X} dst={:#010X} count={} size={} {}{}",
            ch,
            src,
            dst,
            count,
            size,
            if is_rom { "[ROM→]" } else { "" },
            if is_vram { "[→VRAM]" } else { "" }
        );
    }

    println!("\nAll VRAM DMA transfers:");
    for (ch, src, dst, count, size) in dma_log {
        if *dst >= 0x0600_0000 && *dst < 0x0602_0000 {
            let vram_off = *dst - 0x0600_0000;
            let is_rom = *src >= 0x0800_0000;
            println!(
                "  DMA{}: src={:#010X} dst={:#010X}(VRAM+{:#X}) count={} size={} {}",
                ch,
                src,
                dst,
                vram_off,
                count,
                size,
                if is_rom { "[ROM→VRAM]" } else { "[?→VRAM]" }
            );
        }
    }
}
