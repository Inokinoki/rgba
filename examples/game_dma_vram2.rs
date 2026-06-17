use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.dma_log_enabled = true;
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..100 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();
    let dma_log = &gba.mem.dma_log;

    println!("After 100 frames:");
    println!("  DMA log: {} entries", dma_log.len());
    println!("  VRAM write log: {} entries", gba.mem.vram_write_log.len());

    // DMA transfers to VRAM
    let mut vram_dma = 0;
    let mut tile_dma = 0;
    for &(ch, src, dst, count, size) in dma_log {
        if dst >= 0x0600_0000 && dst < 0x0618_0000 {
            vram_dma += 1;
            let dst_off = (dst - 0x0600_0000) as usize;
            if dst_off < 0x10000 {
                tile_dma += 1;
            }
            if vram_dma <= 10 {
                println!(
                    "  DMA{}: {:#010X}→{:#010X} count={} size={} total_bytes={}",
                    ch,
                    src,
                    dst,
                    count,
                    size,
                    count * size
                );
            }
        }
    }
    println!(
        "Total DMA to VRAM: {}, to BG tile area: {}",
        vram_dma, tile_dma
    );

    // Check tile 394 after DMA
    let tile_off = 394 * 32;
    let mut has_data = false;
    for i in 0..32 {
        if vram[tile_off + i] != 0 {
            has_data = true;
            break;
        }
    }
    println!("\nTile 394 has data: {}", has_data);
    if has_data {
        for row in 0..8 {
            let off = tile_off + row * 4;
            print!("  Row {}: ", row);
            for b in 0..4 {
                print!("{:02X}", vram[off + b]);
            }
            println!();
        }
    }

    // Check first DMA transfer in detail
    if !dma_log.is_empty() {
        println!("\nFirst 5 DMA transfers:");
        for (i, &(ch, src, dst, count, size)) in dma_log.iter().take(5).enumerate() {
            println!(
                "  [{}] DMA{}: {:#010X}→{:#010X} count={} size={}",
                i, ch, src, dst, count, size
            );
        }
    }
}
