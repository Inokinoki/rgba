use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().dma_log_enabled = true;

    for frame in 0..200 {
        for _sl in 0..228 {
            gba.run_scanline();
        }
        if frame % 50 == 0 {
            eprintln!("Frame {}", frame);
        }
    }

    let log = &gba.mem().dma_log;
    eprintln!("Total DMA transfers: {}", log.len());

    let mut vram_dmas: Vec<_> = log
        .iter()
        .filter(|(_, src, dst, _, _)| {
            (*dst >= 0x06000000 && *dst < 0x06010000) || (*src >= 0x06000000 && *src < 0x06010000)
        })
        .collect();

    println!("DMA transfers involving VRAM: {}", vram_dmas.len());
    for (ch, src, dst, cnt, ctrl) in vram_dmas.iter().take(30) {
        let dst_region = if *dst >= 0x06000000 && *dst < 0x06010000 {
            "VRAM"
        } else if *dst >= 0x02000000 && *dst < 0x02040000 {
            "EWRAM"
        } else {
            "OTHER"
        };
        println!(
            "  DMA{} src={:08X} dst={:08X} cnt={} ctrl={:08X} ->{}",
            ch, src, dst, cnt, ctrl, dst_region
        );
    }

    let mut ewram_to_vram: Vec<_> = log
        .iter()
        .filter(|(_, src, dst, _, _)| {
            (*src >= 0x02000000 && *src < 0x02040000) && (*dst >= 0x06000000 && *dst < 0x06010000)
        })
        .collect();
    println!("\nEWRAM->VRAM transfers: {}", ewram_to_vram.len());
    for (ch, src, dst, cnt, ctrl) in ewram_to_vram.iter().take(10) {
        println!("  DMA{} src={:08X} dst={:08X} cnt={}", ch, src, dst, cnt);
    }

    let mut rom_to_vram: Vec<_> = log
        .iter()
        .filter(|(_, src, dst, _, _)| {
            (*src >= 0x08000000) && (*dst >= 0x06000000 && *dst < 0x06010000)
        })
        .collect();
    println!("\nROM->VRAM transfers: {}", rom_to_vram.len());
    for (ch, src, dst, cnt, ctrl) in rom_to_vram.iter().take(10) {
        println!("  DMA{} src={:08X} dst={:08X} cnt={}", ch, src, dst, cnt);
    }
}
