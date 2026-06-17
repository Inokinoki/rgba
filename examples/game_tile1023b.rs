use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 1..=20 {
        gba.run_frame();
        let vram = gba.mem().vram();
        let tile1023_off = 1023 * 32;
        let sum: u32 = vram[tile1023_off..tile1023_off + 32]
            .iter()
            .map(|&b| b as u32)
            .sum();
        if sum > 0 {
            println!("Frame {}: tile 1023 sum={}", frame, sum);
            break;
        }
    }

    // Check how VRAM was written
    let log = &gba.mem.vram_write_log;

    // Find what address maps to tile 1023
    // Tile 1023 = offset 0x7FE0 (1023*32 = 32736 = 0x7FE0)
    // In the map_address function:
    // 0x0600_7FE0: raw = 0x7FE0, raw < 0x18000, offset = 0x7FE0
    // That maps to VRAM[0x7FE0]

    // But also check mirror: 0x0600_FFE0: raw = 0xFFE0, raw >= 0x18000, offset = 0xFFE0 - 0x8000 = 0x7FE0
    // So 0x0600_FFE0 also maps to the SAME tile 1023!

    // And: 0x0601_7FE0: raw = 0x17FE0, raw >= 0x18000, offset = 0x17FE0 - 0x8000 = 0xFFE0
    // Wait, 0x17FE0 >= 0x18000? No! 0x17FE0 < 0x18000. So offset = 0x17FE0
    // That's in OBJ VRAM area (0x10000+), not BG tile area
    // But the PPU reads tile data from offset 0x17FE0 if tile_num is large enough

    // Let me check: are writes going to 0x0600_FFE0 (which mirrors to tile 1023)?
    let mut mirror_writes = 0;
    for &(addr, pc, val) in log.iter() {
        let offset = (addr & 0x1FFFF) as usize;
        if offset == 0x7FE0 || offset == 0xFFE0 {
            mirror_writes += 1;
            if mirror_writes <= 5 {
                println!(
                    "Write to tile 1023 area: addr={:#010X} offset={:#06X} val={:#04X} pc={:#010X}",
                    addr,
                    offset,
                    val,
                    pc << 1
                );
            }
        }
    }
    println!("Writes to tile 1023 offset: {}", mirror_writes);

    // Check what addresses in range 0x7F00-0x8000 were written
    println!("\nWrites to 0x7F00-0x8000 range:");
    for &(addr, pc, val) in log.iter() {
        let offset = (addr & 0x1FFFF) as usize;
        if offset >= 0x7F00 && offset < 0x8000 && val != 0 {
            println!(
                "  offset={:#06X} val={:#04X} pc={:#010X}",
                offset,
                val,
                pc << 1
            );
        }
    }

    // Check 0xFF00-0x10000 range (mirror of 0x7F00-0x8000)
    println!("\nWrites to 0xFF00-0x10000 range:");
    for &(addr, pc, val) in log.iter() {
        let offset = (addr & 0x1FFFF) as usize;
        if offset >= 0xFF00 && offset < 0x10000 && val != 0 {
            println!(
                "  offset={:#06X} val={:#04X} pc={:#010X}",
                offset,
                val,
                pc << 1
            );
        }
    }
}
