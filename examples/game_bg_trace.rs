use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..1000 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let vram = gba.ppu().vram();
    let pal = gba.mem().palette();

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);

    // BG0 setup: screen_base=0xC000, char_base=0x0000, size=1 (64x32)
    let screen_base: usize = 0xC000;
    let char_base: usize = 0x0000;

    // Pick a pixel that should have BG data
    // Screen entry[1] = tile 394 at position (8,0)
    // Let's check pixel (8, 0) - should be in tile 394

    let test_x: u16 = 8;
    let test_y: u16 = 0;

    // Compute screen entry index for 64x32 map
    let map_x = test_x / 8;
    let map_y = test_y / 8;
    let entry_idx = map_y as usize * 64 + map_x as usize;
    let entry_off = screen_base + entry_idx * 2;
    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
    let tile_num = (entry & 0x3FF) as usize;
    let hflip = (entry >> 10) & 1;
    let vflip = (entry >> 11) & 1;
    let pal_bank = ((entry >> 12) & 0xF) as usize;

    println!("Test pixel ({}, {})", test_x, test_y);
    println!(
        "Screen entry[{}]: {:#06X} tile={} hflip={} vflip={} pal_bank={}",
        entry_idx, entry, tile_num, hflip, vflip, pal_bank
    );

    // Get tile pixel data
    let tile_x = (test_x % 8) as usize;
    let tile_y = (test_y % 8) as usize;
    let tile_data_off = char_base + tile_num * 32 + tile_y * 4;
    let tile_byte = vram[tile_data_off];

    // 4bpp: each byte has 2 pixels
    let pixel_idx = tile_x;
    let color_idx = if pixel_idx % 2 == 0 {
        tile_byte & 0x0F
    } else {
        (tile_byte >> 4) & 0x0F
    };

    println!(
        "Tile data byte at off {}: {:#04X}",
        tile_data_off, tile_byte
    );
    println!(
        "Pixel ({}, {}) color_idx: {} (pal_bank={})",
        tile_x, tile_y, color_idx, pal_bank
    );

    // Get actual palette color
    let pal_idx = pal_bank * 16 + color_idx as usize;
    let color = u16::from_le_bytes([pal[pal_idx * 2], pal[pal_idx * 2 + 1]]);
    let r = (color & 0x1F) as u32 * 255 / 31;
    let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
    let b = ((color >> 10) as u32) * 255 / 31;
    println!(
        "Palette color[{}]: {:#06X} rgb=({},{},{})",
        pal_idx, color, r, g, b
    );

    // Now check what get_pixel_tile_mode returns
    let result = gba.get_pixel_tile_mode(test_x, test_y);
    println!(
        "\nget_pixel_tile_mode({}, {}) = {:#06X}",
        test_x, test_y, result
    );

    // Also check several more pixels
    println!("\n=== Sample pixels ===");
    let samples = [
        (0u16, 0u16),
        (8, 0),
        (16, 0),
        (24, 0),
        (32, 0),
        (80, 80),
        (120, 80),
        (160, 80),
    ];
    for (sx, sy) in samples {
        let c = gba.get_pixel_tile_mode(sx, sy);
        let r = (c & 0x1F) as u32 * 255 / 31;
        let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
        let b = ((c >> 10) as u32) * 255 / 31;

        // Also compute manually
        let map_x2 = sx / 8;
        let map_y2 = sy / 8;
        let entry_idx2 = map_y2 as usize * 64 + map_x2 as usize;
        let entry_off2 = screen_base + entry_idx2 * 2;
        let entry2 = u16::from_le_bytes([vram[entry_off2], vram[entry_off2 + 1]]);
        let tile2 = entry2 & 0x3FF;

        println!(
            "  ({:3},{:3}): get_pixel={:#06X} rgb=({:3},{:3},{:3}) tile={}",
            sx, sy, c, r, g, b, tile2
        );
    }

    // Dump raw BG0 tilemap (first row, 64 entries)
    println!("\n=== BG0 tilemap row 0 ===");
    for i in 0..64 {
        let off = screen_base + i * 2;
        let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
        let t = e & 0x3FF;
        if t != 0x3FF {
            print!("{:4}", t);
        } else {
            print!("   .");
        }
        if (i + 1) % 32 == 0 {
            println!();
        }
    }

    // Now trace through get_bg_pixel manually
    println!("\n=== Manual trace of get_bg_pixel for BG0 at (8,0) ===");

    // Check what BG registers are in the snapshot
    let bgcnt_off = 0x08;
    let bg0cnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
    let bg0_char_base = ((bg0cnt >> 2) & 0xF) as u32 * 0x4000;
    let bg0_screen_base = ((bg0cnt >> 8) & 0x1F) as u32 * 0x800;
    let bg0_size = (bg0cnt >> 14) & 3;
    println!(
        "BG0CNT: {:#06X} char_base={:#06X} screen_base={:#06X} size={}",
        bg0cnt, bg0_char_base, bg0_screen_base, bg0_size
    );
    println!("DISPCNT: {:#06X} mode={}", dispcnt, dispcnt & 7);
}
