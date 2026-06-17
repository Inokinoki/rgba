use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Enable VRAM logging
    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for frame in 0..200 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    // Count writes that target char block 0 (tiles 0-511, offset 0x0000-0x3FFF)
    let log = &gba.mem().vram_write_log;
    let cb0_writes: Vec<_> = log
        .iter()
        .filter(|(addr, _, _)| {
            let off = addr - 0x06000000;
            off < 0x4000 // Char block 0
        })
        .collect();

    println!("Total char block 0 writes: {}", cb0_writes.len());

    // Show which offsets in char block 0 were written
    let mut offsets = std::collections::BTreeSet::new();
    for &(addr, pc, val) in &cb0_writes {
        offsets.insert(addr - 0x06000000);
    }

    let max_offset = offsets.iter().max().unwrap_or(&0);
    println!(
        "Max offset written: 0x{:04X} (tile {})",
        max_offset,
        max_offset / 32
    );

    // Check what VRAM actually contains
    let vram = gba.mem().vram();
    println!("\nChar block 0 content check:");
    let mut nonzero_count = 0;
    for tile in 0..512 {
        let offset = tile * 32;
        let nonzero: usize = vram[offset..offset + 32]
            .iter()
            .filter(|&&b| b != 0)
            .count();
        if nonzero > 0 {
            nonzero_count += 1;
            if nonzero_count <= 20 {
                println!(
                    "  Tile {}: {}/32 nonzero bytes, first: {:02X?}",
                    tile,
                    nonzero,
                    &vram[offset..offset + 8.min(32)]
                );
            }
        }
    }
    println!("Total nonzero tiles: {}/512", nonzero_count);

    // Check tile 187 specifically (first tile referenced by BG0 that has no data)
    println!(
        "\nTile 187 content: {:02X?}",
        &vram[187 * 32..187 * 32 + 32]
    );
}
