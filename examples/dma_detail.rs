use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;

    for _ in 0..300u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().dma_log_enabled = false;

    let log = &gba.mem().dma_log;
    println!("Total DMA entries: {}", log.len());

    println!("\n=== All DMA transfers TO VRAM ===");
    let vram_dmas: Vec<_> = log
        .iter()
        .enumerate()
        .filter(|(_, (_, _, dst, _, _))| *dst >= 0x06000000 && *dst < 0x06018000)
        .collect();
    for (idx, (ch, src, dst, count, size)) in &vram_dmas {
        println!(
            "[{}] DMA{}: {:#010X} -> {:#010X} count={} size={}",
            idx, ch, src, dst, count, size
        );
    }
    println!("Total VRAM DMA transfers: {}", vram_dmas.len());

    println!("\n=== DMA transfers by channel ===");
    for ch in 0..4 {
        let count = log.iter().filter(|(c, _, _, _, _)| *c == ch as u8).count();
        println!("DMA{}: {} transfers", ch, count);
    }

    println!("\n=== DMA transfers by destination ===");
    let ewram = log
        .iter()
        .filter(|(_, _, dst, _, _)| *dst >= 0x02000000 && *dst < 0x03000000)
        .count();
    let iwram = log
        .iter()
        .filter(|(_, _, dst, _, _)| *dst >= 0x03000000 && *dst < 0x04000000)
        .count();
    let vram = log
        .iter()
        .filter(|(_, _, dst, _, _)| *dst >= 0x06000000 && *dst < 0x06018000)
        .count();
    let oam = log
        .iter()
        .filter(|(_, _, dst, _, _)| *dst >= 0x07000000 && *dst < 0x07000400)
        .count();
    let io = log
        .iter()
        .filter(|(_, _, dst, _, _)| *dst >= 0x04000000 && *dst < 0x05000000)
        .count();
    println!(
        "EWRAM: {} IWRAM: {} VRAM: {} OAM: {} IO: {}",
        ewram, iwram, vram, oam, io
    );

    println!("\n=== Unique DMA patterns (first 200) ===");
    let mut seen = std::collections::HashSet::new();
    for (ch, src, dst, count, size) in log.iter().take(200) {
        let key = (*ch, (*src) & 0xFF000000, (*dst) & 0xFF000000, *count, *size);
        if seen.insert(key) {
            println!(
                "DMA{}: {:#010X}.. -> {:#010X}.. count={} size={}",
                ch,
                src & 0xFF000000,
                dst & 0xFF000000,
                count,
                size
            );
        }
    }
}
