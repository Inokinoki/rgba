use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..100 {
        gba.run_frame();
    }

    let log = &gba.mem.vram_write_log;
    println!("VRAM write log: {} entries", log.len());

    if log.is_empty() {
        println!("No VRAM writes logged!");
        return;
    }

    // Analyze address distribution
    let mut ranges = [0usize; 8];
    for &(addr, _pc, _val) in log {
        let offset = (addr & 0x1FFFF) as usize;
        match offset {
            0x0000..=0x1FFF => ranges[0] += 1,
            0x2000..=0x3FFF => ranges[1] += 1,
            0x4000..=0x5FFF => ranges[2] += 1,
            0x6000..=0x7FFF => ranges[3] += 1,
            0x8000..=0x9FFF => ranges[4] += 1,
            0xA000..=0xBFFF => ranges[5] += 1,
            0xC000..=0xDFFF => ranges[6] += 1,
            0xE000..=0xFFFF => ranges[7] += 1,
            _ => {}
        }
    }

    println!("\nVRAM write address distribution:");
    let names = [
        "0x0000-0x1FFF (Tile 0-255)",
        "0x2000-0x3FFF (Tile 256-511)",
        "0x4000-0x5FFF (Tile 512-767)",
        "0x6000-0x7FFF (Tile 768-1023)",
        "0x8000-0x9FFF (Tile 1024+ / mirror)",
        "0xA000-0xBFFF (OBJ / Page 1)",
        "0xC000-0xDFFF (Screen entries)",
        "0xE000-0xFFFF (Screen entries)",
    ];
    for (i, name) in names.iter().enumerate() {
        println!("  {}: {} writes", name, ranges[i]);
    }

    // Show first 30 writes
    println!("\nFirst 30 VRAM writes:");
    for (i, &(addr, pc, val)) in log.iter().take(30).enumerate() {
        println!(
            "  [{}] addr={:#010X} val={:#04X} pc={:#010X}",
            i, addr, val, pc
        );
    }

    // Check if any writes go to BG0 tile area (tiles 187-611 → 0x1700-0x4C60)
    let mut bg0_tile_writes = 0;
    let mut min_tile_addr = u32::MAX;
    let mut max_tile_addr = 0u32;
    for &(addr, _pc, _val) in log {
        let offset = addr & 0x1FFFF;
        if offset >= 0x1700 && offset <= 0x4C60 {
            bg0_tile_writes += 1;
            min_tile_addr = min_tile_addr.min(offset);
            max_tile_addr = max_tile_addr.max(offset);
        }
    }
    println!(
        "\nWrites to BG0 tile range (0x1700-0x4C60): {}",
        bg0_tile_writes
    );
    if bg0_tile_writes > 0 {
        println!("  Range: {:#06X} - {:#06X}", min_tile_addr, max_tile_addr);
    }
}
