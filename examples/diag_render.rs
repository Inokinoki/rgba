use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    for _ in 0..1000 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let dc = gba.ppu().get_dispcnt();
    println!("=== Frame 1000 state ===");
    println!(
        "DISPCNT: {:#06X} (mode={} BG_enable={:#04X} OBJ={})",
        dc,
        dc & 7,
        (dc >> 8) & 0xF,
        (dc >> 12) & 1
    );

    for bg in 0..4 {
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let enabled = gba.ppu().is_bg_enabled(bg);
        let priority = bgcnt & 3;
        let char_base = ((bgcnt >> 2) & 3) * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        let is_8bpp = (bgcnt & 0x80) != 0;
        let hofs = gba.ppu().get_bg_hofs(bg);
        let vofs = gba.ppu().get_bg_vofs(bg);
        println!("  BG{}: enabled={} BGCTL={:#06X} pri={} char_base={:#06X} screen_base={:#06X} size={} 8bpp={} hofs={} vofs={}",
            bg, enabled, bgcnt, priority, char_base, screen_base, size, is_8bpp, hofs, vofs);
    }

    println!("\n=== Palette (BG, first 32 colors) ===");
    let palette = gba.mem().palette();
    for i in 0..32 {
        let offset = i * 2;
        let color = u16::from_le_bytes([palette[offset], palette[offset + 1]]);
        let r = color & 0x1F;
        let g = (color >> 5) & 0x1F;
        let b = (color >> 10) & 0x1F;
        if color != 0 {
            println!("  color[{}] = {:#06X} (r={} g={} b={})", i, color, r, g, b);
        }
    }
    println!(
        "  color[0] = {:#06X} (backdrop)",
        u16::from_le_bytes([palette[0], palette[1]])
    );

    let vram = gba.mem().vram();

    let dc_mode = dc & 7;
    for bg in 0..4 {
        if !gba.ppu().is_bg_enabled(bg) {
            continue;
        }
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let screen_base = (((bgcnt >> 8) & 0x1F) * 0x800) as usize;
        let char_base = (((bgcnt >> 2) & 3) * 0x4000) as usize;

        println!("\n=== BG{}: screen entries at {:#06X} ===", bg, screen_base);
        let mut tile_counts = std::collections::HashMap::new();
        for row in 0..32usize {
            for col in 0..32usize {
                let offset = screen_base + (row * 32 + col) * 2;
                if offset + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
                    *tile_counts.entry(entry & 0x3FF).or_insert(0usize) += 1;
                }
            }
        }
        let mut tiles: Vec<_> = tile_counts.iter().collect();
        tiles.sort_by(|a, b| b.1.cmp(a.1));
        for (tile, count) in tiles.iter().take(10) {
            println!("  tile {} appears {} times", tile, count);
        }

        println!("  First 4 rows of screen entries:");
        for row in 0..4usize {
            print!("    row {}: ", row);
            for col in 0..16usize {
                let offset = screen_base + (row * 32 + col) * 2;
                if offset + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
                    print!("{:03X} ", entry & 0x3FF);
                }
            }
            println!();
        }

        println!(
            "  Tile 0 data (first 32 bytes at char_base={:#06X}):",
            char_base
        );
        for row in 0..4usize {
            print!("    ");
            for col in 0..8usize {
                let offset = char_base + row * 8 + col;
                print!("{:02X} ", vram[offset]);
            }
            println!();
        }
    }

    println!("\n=== Manual pixel trace at (120, 80) ===");
    let x: u16 = 120;
    let y: u16 = 80;
    for bg in 0..4 {
        if !gba.ppu().is_bg_enabled(bg) {
            continue;
        }
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let screen_base = (((bgcnt >> 8) & 0x1F) * 0x800) as usize;
        let char_base = (((bgcnt >> 2) & 0x3) * 0x4000) as usize;
        let hofs = gba.ppu().get_bg_hofs(bg);
        let vofs = gba.ppu().get_bg_vofs(bg);
        let bg_x = (x as u32 + hofs as u32) % 256;
        let bg_y = (y as u32 + vofs as u32) % 256;
        let tile_x = bg_x as usize / 8;
        let tile_y = bg_y as usize / 8;
        let pixel_x = bg_x as usize % 8;
        let pixel_y = bg_y as usize % 8;
        let entry_offset = screen_base + (tile_y * 32 + tile_x) * 2;
        let entry = if entry_offset + 1 < vram.len() {
            u16::from_le_bytes([vram[entry_offset], vram[entry_offset + 1]])
        } else {
            0
        };
        let tile_num = entry & 0x3FF;
        let palette_num = (entry >> 12) & 0xF;
        let tile_data_offset = char_base + tile_num as usize * 32;
        let color_index = if tile_data_offset + pixel_y * 4 + pixel_x / 2 < vram.len() {
            let b = vram[tile_data_offset + pixel_y * 4 + pixel_x / 2];
            if pixel_x % 2 == 0 {
                b & 0x0F
            } else {
                b >> 4
            }
        } else {
            0
        };
        let color = gba.get_palette_color(0, palette_num * 16 + color_index as u16);
        println!(
            "  BG{}: hofs={} vofs={} bg_pos=({}, {}) tile=({}, {}) px=({}, {})",
            bg, hofs, vofs, bg_x, bg_y, tile_x, tile_y, pixel_x, pixel_y
        );
        println!(
            "    entry@{:#06X}={:#06X} tile_num={} pal={} color_idx={} pal_color={:#06X}",
            entry_offset, entry, tile_num, palette_num, color_index, color
        );
        if let Some(bg_color) = gba.get_bg_pixel(gba.ppu(), dc_mode as u8, bg, x, y) {
            println!("    get_bg_pixel -> Some({:#06X})", bg_color);
        } else {
            println!("    get_bg_pixel -> None (transparent)");
        }
    }

    println!("\n=== get_pixel_tile_mode samples ===");
    for y in [0u16, 40, 80, 120, 159] {
        for x in [0u16, 60, 120, 180, 239] {
            let c = gba.get_pixel_tile_mode(x, y);
            let r = (c & 0x1F) as u32 * 255 / 31;
            let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
            let b = ((c >> 10) as u32) * 255 / 31;
            print!(" ({},{})={:#06X}[{},{},{}]", x, y, c, r, g, b);
        }
        println!();
    }

    let mut white_count = 0u32;
    let mut black_count = 0u32;
    let mut other_count = 0u32;
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = gba.get_pixel_tile_mode(x, y);
            if c == 0 {
                black_count += 1;
            } else if c == 0x7FFF {
                white_count += 1;
            } else {
                other_count += 1;
            }
        }
    }
    println!("\n=== Pixel counts ===");
    println!(
        "Black(0): {}, White(0x7FFF): {}, Other: {}",
        black_count, white_count, other_count
    );
}
