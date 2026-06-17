use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;

    for frame in 0..300u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().dma_log_enabled = false;

    let log = &gba.mem().dma_log;
    println!("DMA log entries: {}", log.len());

    for (ch, src, dst, count, size) in log.iter().take(50) {
        let src_region = if *src >= 0x08000000 && *src < 0x0A000000 {
            "ROM"
        } else if *src >= 0x02000000 && *src < 0x03000000 {
            "EWRAM"
        } else if *src >= 0x03000000 && *src < 0x04000000 {
            "IWRAM"
        } else if *src >= 0x06000000 && *src < 0x06018000 {
            "VRAM"
        } else if *src >= 0x00000000 && *src < 0x00004000 {
            "BIOS"
        } else {
            "???"
        };
        let dst_region = if *dst >= 0x06000000 && *dst < 0x06018000 {
            "VRAM"
        } else if *dst >= 0x02000000 && *dst < 0x03000000 {
            "EWRAM"
        } else if *dst >= 0x03000000 && *dst < 0x04000000 {
            "IWRAM"
        } else if *dst >= 0x08000000 && *dst < 0x0A000000 {
            "ROM"
        } else {
            "???"
        };
        println!(
            "DMA{}: {:#010X}({}) -> {:#010X}({}) count={} size={}",
            ch, src, src_region, dst, dst_region, count, size
        );
    }

    if log.len() > 50 {
        println!("... {} more entries", log.len() - 50);
    }

    let vram_transfers = log
        .iter()
        .filter(|(_, _, dst, _, _)| *dst >= 0x06000000 && *dst < 0x06018000)
        .count();
    println!("\nDMA transfers TO VRAM: {}", vram_transfers);

    let rom_transfers = log
        .iter()
        .filter(|(_, src, dst, _, _)| *src >= 0x08000000 && *dst >= 0x06000000 && *dst < 0x06018000)
        .count();
    println!("DMA transfers ROM -> VRAM: {}", rom_transfers);

    println!("\n=== DMA channel states ===");
    for i in 0..4 {
        let dma = &gba.dma[i];
        println!(
            "DMA{}: enabled={} active={} src={:#010X} dst={:#010X} count={} trigger={:?} repeat={}",
            i,
            dma.is_enabled(),
            dma.is_active(),
            dma.get_src_addr(),
            dma.get_dst_addr(),
            dma.get_count(),
            dma.get_trigger(),
            dma.is_repeat()
        );
    }

    println!("\n=== Key IWRAM values (game state) ===");
    for off in (0x0900..0x0980).step_by(4) {
        let val = gba.mem_read_word(0x03000000 + off as u32);
        if val != 0 {
            println!("  IWRAM[{:#06X}] = {:#010X}", off, val);
        }
    }
}
