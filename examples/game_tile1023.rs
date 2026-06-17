use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Check VRAM before any execution
    let vram = gba.mem().vram();
    let tile1023_off = 1023 * 32;
    let mut sum = 0u32;
    for i in 0..32 {
        sum += vram[tile1023_off + i] as u32;
    }
    println!("Before execution: tile 1023 sum={}", sum);

    // Run 3 frames (when data first appears)
    for _ in 0..3 {
        gba.run_frame();
    }

    let vram = gba.mem().vram();
    let mut sum = 0u32;
    for i in 0..32 {
        sum += vram[tile1023_off + i] as u32;
    }
    println!("After frame 3: tile 1023 sum={}", sum);

    // Check if any VRAM write went to tile 1023
    let log = &gba.mem.vram_write_log;
    let mut writes_to_1023 = 0;
    for &(addr, _pc, val) in log {
        let offset = (addr & 0x1FFFF) as usize;
        if offset >= tile1023_off && offset < tile1023_off + 32 {
            writes_to_1023 += 1;
            if writes_to_1023 <= 5 {
                println!(
                    "  Write to tile 1023: offset={:#06X} val={:#04X}",
                    offset, val
                );
            }
        }
    }
    println!("Writes to tile 1023: {}", writes_to_1023);

    // Check if tile 1023 data comes from VRAM mirror
    // VRAM mirror: 0x18000-0x1FFFF mirrors 0x10000-0x17FFF (OBJ)
    // Write to 0x18000 might mirror to 0x10000+?
    // Or: DMA to 0x6001_0000 (OBJ area) might mirror to BG area?

    // Let me check: where does 0xFF data come from?
    // If a DMA writes 0xFF to VRAM, where does it go?

    let dma_entries: Vec<(u8, u32, u32, u32, u32)> = gba.mem.dma_log.clone();
    drop(gba);

    for (ch, src, dst, count, _size) in dma_entries.iter().take(20) {
        let dst_off = (dst - 0x0600_0000) as usize;
        if dst_off >= 0xF800 && dst_off < 0x10000 {
            println!(
                "DMA{} to screen area: {:#010X}→{:#010X} (off={:#06X}) count={}",
                ch, src, dst, dst_off, count
            );
        }
    }
}
