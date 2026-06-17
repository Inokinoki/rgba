use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Check VRAM at frame 0 (before any emulation)
    let vram_init = gba.mem().vram().to_vec();

    // Also check what initial VRAM looks like (should be all zeros)
    let init_nonzero: usize = vram_init.iter().filter(|&&b| b != 0).count();
    eprintln!("Initial VRAM nonzero bytes: {}", init_nonzero);

    // Run 10 frames
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..10 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    let vram_10 = gba.mem().vram().to_vec();
    let nonzero_10: usize = vram_10.iter().filter(|&&b| b != 0).count();
    eprintln!("After 10 frames VRAM nonzero bytes: {}", nonzero_10);

    // Find which regions changed
    println!("\nVRAM changes by region:");
    for region in 0..6 {
        let start = region * 0x4000;
        let end = (region + 1) * 0x4000;
        let mut changed = 0;
        let mut nonzero = 0;
        for i in start..end {
            if vram_10[i] != vram_init[i] {
                changed += 1;
            }
            if vram_10[i] != 0 {
                nonzero += 1;
            }
        }
        println!(
            "  {:05X}-{:05X}: {} bytes changed, {} nonzero",
            start,
            end - 1,
            changed,
            nonzero
        );
    }

    // Check tile data region specifically
    println!("\nTile data (0x0000-0x8000) nonzero by 0x400-byte blocks:");
    for block in 0..32 {
        let start = block * 0x400;
        let end = start + 0x400;
        let nonzero: usize = vram_10[start..end].iter().filter(|&&b| b != 0).count();
        if nonzero > 0 {
            println!("  {:05X}-{:05X}: {} nonzero bytes", start, end - 1, nonzero);
        }
    }

    // Run to frame 240 and check again
    for _ in 10..240 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    let vram_240 = gba.mem().vram().to_vec();
    let nonzero_240: usize = vram_240.iter().filter(|&&b| b != 0).count();
    eprintln!("After 240 frames VRAM nonzero bytes: {}", nonzero_240);

    println!("\nAfter 240 frames - VRAM changes since frame 10:");
    for region in 0..6 {
        let start = region * 0x4000;
        let end = (region + 1) * 0x4000;
        let mut changed = 0;
        for i in start..end {
            if vram_240[i] != vram_10[i] {
                changed += 1;
            }
        }
        if changed > 0 {
            println!("  {:05X}-{:05X}: {} bytes changed", start, end - 1, changed);
        }
    }

    // Check if tile data was written after frame 10
    let tile_changed: usize = (0x0000..0x8000)
        .filter(|&i| vram_240[i] != vram_10[i])
        .count();
    println!(
        "\nTile data (0x0000-0x8000) changed between frame 10 and 240: {} bytes",
        tile_changed
    );
}
