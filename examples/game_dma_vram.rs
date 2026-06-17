use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..100 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();
    let log = &gba.mem.vram_write_log;

    println!("After 100 frames: {} VRAM writes logged", log.len());

    // Find the MAX VRAM address that has non-zero data
    let mut max_nonzero = 0usize;
    for i in 0..0x10000 {
        if vram[i] != 0 {
            max_nonzero = i;
        }
    }
    println!("Max non-zero VRAM address (BG): {:#06X}", max_nonzero);
    println!("That's tile {} byte {}", max_nonzero / 32, max_nonzero % 32);

    // Check specific addresses: tile 187 (first BG0 tile)
    let tile187_off = 187 * 32;
    println!("\nTile 187 (offset {:#06X}):", tile187_off);
    for row in 0..8 {
        let off = tile187_off + row * 4;
        print!("  Row {}: ", row);
        for b in 0..4 {
            print!("{:02X} ", vram[off + b]);
        }
        println!();
    }

    // Check if vram_log has entries for tile area
    let mut tile_log_entries = 0;
    for &(addr, _pc, val) in log {
        let offset = (addr & 0x1FFFF) as usize;
        if offset < 0x4000 && val != 0 {
            tile_log_entries += 1;
        }
    }
    println!("\nNon-zero VRAM writes to tile area: {}", tile_log_entries);

    // Check DMA log
    let dma_log = &gba.mem.dma_log;
    println!("\nDMA log: {} entries", dma_log.len());

    // Find any DMA to VRAM
    let mut vram_dma = 0;
    for &(ch, src, dst, count, size) in dma_log {
        let dst_offset = (dst & 0x1FFFF) as usize;
        if dst >= 0x0600_0000 && dst < 0x0610_0000 && dst_offset < 0x4000 {
            vram_dma += 1;
            if vram_dma <= 5 {
                println!(
                    "  DMA{}: {:#010X}→{:#010X} count={} size={}",
                    ch, src, dst, count, size
                );
            }
        }
    }
    println!("DMA transfers to VRAM tile area: {}", vram_dma);

    // Check write_word to VRAM
    let mut word_vram = 0;
    for &(addr, _pc, _val) in log {
        let offset = addr & 0x1FFFF;
        if offset >= 0x1760 && offset < 0x4000 {
            word_vram += 1;
        }
    }
    println!("VRAM writes to 0x1760-0x3FFF: {}", word_vram);

    // The game has tile data. Where did it come from?
    // Let me check if VRAM was initialized with data
    // Gba::new() might initialize VRAM with 0xFF
    let mut gba2 = Gba::new();
    let vram2 = gba2.mem().vram();
    let mut init_nz = 0;
    for &b in vram2.iter() {
        if b != 0 {
            init_nz += 1;
        }
    }
    println!("\nFresh VRAM non-zero bytes: {}/{}", init_nz, vram2.len());

    // Check first few bytes of fresh VRAM
    print!("Fresh VRAM[0..16]: ");
    for i in 0..16 {
        print!("{:02X} ", vram2[i]);
    }
    println!();
}
