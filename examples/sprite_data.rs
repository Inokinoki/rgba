use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.sync_ppu_full();

    let vram = gba.ppu().vram();

    // Tile 784 in 8bpp mode starts at tile_base + 784 * 64
    // For sprites, tile_base is 0x10000 (DISPCNT bit 2 = 0 means tile_base = 0x10000 for OBJ)
    // Wait, DISPCNT = 0x1F40: bits 0-2 = mode 0, bit 6 = OBJ tile mapping (1 = 1D)
    // OBJ tile base for mode 0: 0x10000
    let dc = gba.ppu().get_dispcnt();
    let obj_tile_1d = (dc >> 6) & 1;
    println!(
        "OBJ tile mapping: {} ({})",
        obj_tile_1d,
        if obj_tile_1d == 1 { "1D" } else { "2D" }
    );

    // In 1D mapping, tile 784 starts at 0x10000 + 784 * 32 (for 4bpp) or 784 * 64 (for 8bpp)
    // Check the sprite: pal=7, 256c=1 means 8bpp (256 color mode)
    let tile_addr = 0x10000 + 784 * 64;
    println!("\nTile 784 (8bpp) at VRAM offset 0x{:X}:", tile_addr);

    // Show first 64 bytes (first 8x8 block of the 64x64 sprite)
    let mut has_data = false;
    for row in 0..8 {
        let mut line = String::new();
        for col in 0..8 {
            let off = tile_addr + row * 8 + col;
            let b = vram[off];
            if b != 0 {
                has_data = true;
            }
            line.push_str(&format!("{:02X} ", b));
        }
        println!("  row {}: {}", row, line);
    }
    println!("Has non-zero data: {}", has_data);

    // Also check tile 768 (first visible sprite)
    let tile_addr_768 = 0x10000 + 768 * 64;
    let mut has_data_768 = false;
    for row in 0..8 {
        for col in 0..8 {
            let off = tile_addr_768 + row * 8 + col;
            if vram[off] != 0 {
                has_data_768 = true;
            }
        }
    }
    println!("\nTile 768 has data: {}", has_data_768);

    // Check OBJ palette
    let pal = gba.mem.palette();
    println!("\nOBJ palette banks 7 and 8:");
    for bank in [7, 8] {
        let base = (128 + bank * 16) * 2;
        let mut nz = 0;
        for i in 0..16 {
            let v = u16::from_le_bytes([pal[base + i * 2], pal[base + i * 2 + 1]]);
            if v != 0 {
                nz += 1;
                if nz <= 4 {
                    println!("  PAL[{}][{}]={:04X}", 128 + bank * 16 + i, bank, v);
                }
            }
        }
        println!("  Bank {} non-zero: {}/16", bank, nz);
    }
}
