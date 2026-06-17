use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    for _ in 0..500 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let ppu = gba.ppu();
    let ppu_vram = ppu.vram();
    let mem_vram = gba.mem().vram();

    // Compare PPU VRAM vs Memory VRAM
    let mut diff_count = 0;
    let mut first_diff = 0;
    for i in 0..ppu_vram.len().min(mem_vram.len()) {
        if ppu_vram[i] != mem_vram[i] {
            if diff_count == 0 {
                first_diff = i;
            }
            diff_count += 1;
        }
    }
    println!(
        "VRAM diff: {} / {} bytes differ, first diff at {:#06X}",
        diff_count,
        ppu_vram.len().min(mem_vram.len()),
        first_diff
    );

    // Manual trace of BG3 at (0,0)
    let bg = 3;
    let bgcnt = ppu.get_bgcnt(bg);
    let screen_base = ppu.get_bg_map_base(bg) as usize;
    let char_base = ppu.get_bg_tile_base(bg) as usize;

    println!("\nBG3 at (0,0):");
    println!(
        "  bgcnt={:#06X} screen_base={:#06X} char_base={:#06X}",
        bgcnt, screen_base, char_base
    );

    // Read screen entry at (0,0): tile_x=0, tile_y=0
    let entry_off = screen_base;
    let entry = u16::from_le_bytes([ppu_vram[entry_off], ppu_vram[entry_off + 1]]);
    println!("  Screen entry at {:#06X}: {:#06X}", entry_off, entry);

    let tile_num = entry & 0x3FF;
    let pal_num = (entry >> 12) & 0xF;
    println!("  tile_num={} palette={}", tile_num, pal_num);

    // Read tile data
    let tile_off = char_base + tile_num as usize * 32;
    println!("  Tile data offset: {:#06X}", tile_off);

    // pixel (0,0) in tile: x=0, y=0, 4bpp
    let byte_off = tile_off + 0; // row 0
    let byte_val = ppu_vram[byte_off];
    let color_idx = byte_val & 0xF; // low nibble for x=0
    println!(
        "  Byte at {:#06X}: {:#02X}, color_idx={}",
        byte_off, byte_val, color_idx
    );

    // Palette lookup
    if color_idx != 0 {
        let pal_off = pal_num as usize * 32 + color_idx as usize * 2;
        let pal = gba.mem().palette();
        let color = u16::from_le_bytes([pal[pal_off], pal[pal_off + 1]]);
        let r = color & 0x1F;
        let g = (color >> 5) & 0x1F;
        let b = (color >> 10) & 0x1F;
        println!(
            "  Palette[{}] at off={:#04X}: {:#06X} rgb({},{},{})",
            color_idx, pal_off, color, r, g, b
        );
    }

    // Now get the actual pixel through get_bg_pixel
    if let Some(c) = gba.get_bg_pixel(ppu, 0, bg, 0, 0) {
        let r = c & 0x1F;
        let g = (c >> 5) & 0x1F;
        let b = (c >> 10) & 0x1F;
        println!(
            "  get_bg_pixel(BG3, 0,0) = {:#06X} rgb({},{},{})",
            c, r, g, b
        );
    } else {
        println!("  get_bg_pixel(BG3, 0,0) = None (transparent)");
    }

    // Check PPU vram at tile 1023 offset vs Memory vram
    let t1023_off = 1023 * 32;
    println!("\nTile 1023 data comparison:");
    print!("  PPU:  ");
    for i in 0..8 {
        print!("{:02X} ", ppu_vram[t1023_off + i]);
    }
    println!();
    print!("  MEM:  ");
    for i in 0..8 {
        print!("{:02X} ", mem_vram[t1023_off + i]);
    }
    println!();
}
