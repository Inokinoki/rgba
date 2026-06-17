use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.dma_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..100 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();
    let wram = gba.mem().wram();
    let dma_log = &gba.mem.dma_log;

    // DMA analysis
    println!("DMA log: {} entries", dma_log.len());

    let mut tile_dma = 0;
    let mut screen_dma = 0;
    let mut other_dma = 0;
    for &(ch, src, dst, count, size) in dma_log {
        if dst >= 0x0600_0000 && dst < 0x0618_0000 {
            let off = (dst - 0x0600_0000) as usize;
            if off < 0xC000 {
                tile_dma += 1;
                if tile_dma <= 5 {
                    println!(
                        "  TILE DMA{}: {:#010X}→{:#010X} count={} size={}",
                        ch, src, dst, count, size
                    );
                }
            } else if off < 0x10000 {
                screen_dma += 1;
            } else {
                other_dma += 1;
            }
        }
    }
    println!(
        "DMA to VRAM: tile={}, screen={}, other={}",
        tile_dma, screen_dma, other_dma
    );

    // EWRAM: check if tile data is sitting there
    // Scan EWRAM for data blocks that could be tile data
    // The game might decompress to EWRAM and expect to DMA/CpuSet to VRAM
    // but the transfer never happens

    println!("\n=== VRAM tile data summary ===");
    println!("Max tile with data: {}", {
        let mut max = 0;
        for t in 0..1024 {
            let off = t * 32;
            if off + 32 <= vram.len() && vram[off..off + 32].iter().any(|&b| b != 0) {
                max = t;
            }
        }
        max
    });

    // EWRAM analysis: look for potential tile data
    // In a typical GBA game, tile data is stored compressed in ROM,
    // decompressed to EWRAM, then DMA'd to VRAM
    // If DMA doesn't transfer it, it sits in EWRAM

    // Check if EWRAM has data at offsets that correspond to tile data
    // Tiles 187+ data would need 187*32=0xEB0 to 629*32=0x4E60+32
    let ewram_tile_range: usize = 0x4E80;
    let mut ewram_sum = 0u64;
    for i in 0..ewram_tile_range {
        if i < wram.len() {
            ewram_sum += wram[i] as u64;
        }
    }
    println!("\nEWRAM[0..0x4E80] sum: {}", ewram_sum);

    // Check specific EWRAM offsets for tile-like data
    for offset in [
        0x0000, 0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x10000, 0x20000,
    ] {
        let mut sum = 0u64;
        for i in 0..256 {
            if offset + i < wram.len() {
                sum += wram[offset + i] as u64;
            }
        }
        if sum > 0 {
            println!("  EWRAM[{:#06X}..]: sum={}", offset, sum);
        }
    }

    // Check EWRAM total
    let ewram_total: u64 = wram.iter().map(|&b| b as u64).sum();
    println!("EWRAM total sum: {} / {}KB", ewram_total, wram.len() / 1024);
}
