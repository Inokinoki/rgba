use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().dma_log_enabled = true;
    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..200u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().dma_log_enabled = false;

    let dma_log = &gba.mem().dma_log;
    println!("=== DMA transfers (total: {}) ===", dma_log.len());
    for (ch, src, dst, count, ctrl) in dma_log {
        let src_desc = if *src >= 0x06000000 && *src < 0x06018000 {
            "VRAM"
        } else if *src >= 0x08000000 {
            "ROM"
        } else if *src >= 0x02000000 {
            "EWRAM"
        } else if *src >= 0x03000000 {
            "IWRAM"
        } else {
            "???"
        };
        let dst_desc = if *dst >= 0x06000000 && *dst < 0x06018000 {
            let tile = (*dst - 0x06000000) / 32;
            "VRAM"
        } else if *dst >= 0x02000000 {
            "EWRAM"
        } else if *dst >= 0x03000000 {
            "IWRAM"
        } else {
            "???"
        };
        let repeat = if ctrl & (1 << 9) != 0 { "REP " } else { "" };
        let mode = match (ctrl >> 12) & 3 {
            0 => "Imm",
            1 => "VBlank",
            2 => "HBlank",
            3 => "Special",
            _ => "???",
        };
        let unit = if ctrl & (1 << 10) != 0 { "32" } else { "16" };
        println!(
            "  DMA{}: {} -> {} count={:#X} {}{}{}bit ctrl={:#X}",
            ch,
            format!("{}({:#010X})", src_desc, src),
            format!("{}({:#010X})", dst_desc, dst),
            count,
            repeat,
            mode,
            unit,
            ctrl
        );
    }

    let vram_writes_to_tile: Vec<_> = gba
        .mem()
        .vram_write_log
        .iter()
        .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600C000)
        .collect();
    println!("\n=== VRAM tile writes ===");
    println!("Total: {}", vram_writes_to_tile.len());

    let vram_writes_to_map: Vec<_> = gba
        .mem()
        .vram_write_log
        .iter()
        .filter(|(a, _, _)| *a >= 0x0600C000 && *a < 0x06010000)
        .collect();
    println!("VRAM map writes: {}", vram_writes_to_map.len());

    let vram_writes_to_obj: Vec<_> = gba
        .mem()
        .vram_write_log
        .iter()
        .filter(|(a, _, _)| *a >= 0x06010000)
        .collect();
    println!("VRAM obj writes: {}", vram_writes_to_obj.len());
}
