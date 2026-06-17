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
    let dc = ppu.get_dispcnt();
    let mode = dc & 0x7;
    println!("=== PPU State at F500 ===");
    println!("DISPCNT={:#06X} mode={}", dc, mode);
    println!(
        "BG enabled: BG0={} BG1={} BG2={} BG3={} OBJ={}",
        (dc >> 8) & 1,
        (dc >> 9) & 1,
        (dc >> 10) & 1,
        (dc >> 11) & 1,
        (dc >> 12) & 1
    );

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let priority = bgcnt & 0x3;
        let char_base = (bgcnt >> 2) & 0xF;
        let screen_base = (bgcnt >> 8) & 0x1F;
        let is_8bpp = (bgcnt >> 7) & 1;
        let size = (bgcnt >> 14) & 0x3;
        let enabled = ppu.is_bg_enabled(bg);
        println!("BG{}: cnt={:#06X} pri={} char_base={:#04X} screen_base={:#04X} 8bpp={} size={} enabled={}", 
            bg, bgcnt, priority, char_base * 0x4000, screen_base * 0x800, is_8bpp, size, enabled);

        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        println!("  hofs={} vofs={}", hofs, vofs);
    }

    // Check VRAM content
    let vram = gba.mem().vram();
    let mut nonzero_count = 0;
    let mut first_nonzero = 0;
    for (i, &b) in vram.iter().enumerate() {
        if b != 0 {
            if nonzero_count == 0 {
                first_nonzero = i;
            }
            nonzero_count += 1;
        }
    }
    println!(
        "\nVRAM: {} / {} nonzero bytes ({:.1}%) first_nonzero={:#06X}",
        nonzero_count,
        vram.len(),
        nonzero_count as f64 / vram.len() as f64 * 100.0,
        first_nonzero
    );

    // Check palette
    let pal = gba.mem().palette();
    let mut nonzero_pal = 0;
    for i in (0..pal.len()).step_by(2) {
        let c = u16::from_le_bytes([pal[i], pal[i + 1]]);
        if c != 0 {
            nonzero_pal += 1;
        }
    }
    println!(
        "Palette: {} / {} nonzero entries",
        nonzero_pal,
        pal.len() / 2
    );

    // Try to get BG pixel at center of screen
    println!("\n=== BG pixel test at (120, 80) ===");
    let x = 120u16;
    let y = 80u16;
    for bg in 0..4 {
        if let Some(c) = gba.get_bg_pixel(ppu, mode as u8, bg, x, y) {
            println!("BG{} pixel at ({},{}): {:#06X}", bg, x, y, c);
        } else {
            println!("BG{} pixel at ({},{}): None (transparent)", bg, x, y);
        }
    }

    // Try to manually trace BG0 rendering at (120,80)
    let bg = 0;
    let bgcnt = ppu.get_bgcnt(bg);
    let hofs = ppu.get_bg_hofs(bg);
    let vofs = ppu.get_bg_vofs(bg);
    let bg_x = (x as u32 + hofs as u32) % 256;
    let bg_y = (y as u32 + vofs as u32) % 256;
    let tile_x = bg_x / 8;
    let tile_y = bg_y / 8;
    let pixel_x = (bg_x % 8) as u8;
    let pixel_y = (bg_y % 8) as u8;
    let screen_base = ppu.get_bg_map_base(bg) as usize;
    let char_base = ppu.get_bg_tile_base(bg) as usize;

    println!("\n=== Manual BG0 trace at (120,80) ===");
    println!("hofs={} vofs={} -> bg_x={} bg_y={}", hofs, vofs, bg_x, bg_y);
    println!(
        "tile_x={} tile_y={} pixel_x={} pixel_y={}",
        tile_x, tile_y, pixel_x, pixel_y
    );
    println!(
        "screen_base={:#08X} char_base={:#08X}",
        screen_base, char_base
    );

    // Read screen entry directly from VRAM
    let map_offset = screen_base + (tile_y as usize * 32 + tile_x as usize) * 2;
    if map_offset + 1 < vram.len() {
        let entry = u16::from_le_bytes([vram[map_offset], vram[map_offset + 1]]);
        let tile_num = entry & 0x3FF;
        let flip_h = (entry >> 10) & 1;
        let flip_v = (entry >> 11) & 1;
        let pal_num = (entry >> 12) & 0xF;
        println!(
            "Screen entry at map_offset={:#06X}: {:#06X}",
            map_offset, entry
        );
        println!(
            "  tile_num={} flip_h={} flip_v={} palette={}",
            tile_num, flip_h, flip_v, pal_num
        );

        // Check tile data
        let is_8bpp = (bgcnt & 0x80) != 0;
        if is_8bpp {
            let tile_offset = char_base + tile_num as usize * 64;
            let pixel_offset = tile_offset + (pixel_y as usize * 8 + pixel_x as usize);
            if pixel_offset < vram.len() {
                println!(
                    "  Tile pixel at offset={:#06X}: {:#04X}",
                    pixel_offset, vram[pixel_offset]
                );
            }
        } else {
            let tile_offset = char_base + tile_num as usize * 32;
            let row_offset = tile_offset + (pixel_y as usize * 4);
            let byte_idx = pixel_x as usize / 2;
            let is_high = pixel_x % 2 != 0;
            if row_offset + byte_idx < vram.len() {
                let byte = vram[row_offset + byte_idx];
                let color_idx = if is_high { byte >> 4 } else { byte & 0xF };
                println!(
                    "  Tile pixel at offset={:#06X}: byte={:#04X} color_idx={}",
                    row_offset + byte_idx,
                    byte,
                    color_idx
                );
                if color_idx != 0 {
                    let pal_offset = pal_num as usize * 32 + color_idx as usize * 2;
                    let pal = gba.mem().palette();
                    if pal_offset + 1 < pal.len() {
                        let color = u16::from_le_bytes([pal[pal_offset], pal[pal_offset + 1]]);
                        println!(
                            "  Palette color at offset={:#04X}: {:#06X}",
                            pal_offset, color
                        );
                    }
                }
            }
        }
    }

    // Check how many screen entries are nonzero in BG0 map area
    let bg0_map_base = screen_base;
    let mut nonzero_entries = 0;
    for i in 0..1024 {
        let off = bg0_map_base + i * 2;
        if off + 1 < vram.len() {
            let e = u16::from_le_bytes([vram[off], vram[off + 1]]);
            if e != 0 {
                nonzero_entries += 1;
            }
        }
    }
    println!("\nBG0 screen entries: {} / 1024 nonzero", nonzero_entries);

    // Check if char base area has data
    let char_data = if char_base < vram.len() {
        let mut n = 0;
        for &b in &vram[char_base..vram.len().min(char_base + 0x4000)] {
            if b != 0 {
                n += 1;
            }
        }
        n
    } else {
        0
    };
    println!(
        "BG0 char data: {} nonzero bytes starting at {:#06X}",
        char_data, char_base
    );
}
