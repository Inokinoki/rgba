use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = &gba.ppu;

    println!("=== Mosaic State ===");
    println!("bg_mosaic raw: 0x{:04X}", ppu.bg_mosaic);
    println!(
        "bg_mosaic H: {} (raw {})",
        ppu.get_bg_mosaic_h(),
        ppu.get_bg_mosaic_h_raw()
    );
    println!(
        "bg_mosaic V: {} (raw {})",
        ppu.get_bg_mosaic_v(),
        ppu.get_bg_mosaic_v_raw()
    );
    println!("obj_mosaic raw: 0x{:04X}", ppu.obj_mosaic);

    let bg0cnt = ppu.get_bgcnt(0);
    let mosaic_enabled = (bg0cnt & 0x40) != 0;
    println!("BG0CNT: 0x{:04X}, mosaic bit: {}", bg0cnt, mosaic_enabled);

    let hofs = ppu.get_bg_hofs(0);
    let vofs = ppu.get_bg_vofs(0);
    println!("BG0 hofs={}, vofs={}", hofs, vofs);

    println!("\n=== Mosaic effect on BG0 coordinates ===");
    let mode = ppu.get_display_mode();
    for y in 0..8u16 {
        for x in 0..8u16 {
            let bg_x = ((x as u32 + hofs as u32) % 256) as u16;
            let bg_y = ((y as u32 + vofs as u32) % 256) as u16;
            let (mx, my) = ppu.apply_bg_mosaic(bg_x, bg_y);
            if (mx, my) != (bg_x, bg_y) {
                println!(
                    "  screen({}, {}) -> bg({}, {}) -> mosaic({}, {})",
                    x, y, bg_x, bg_y, mx, my
                );
            }
        }
    }

    println!("\n=== Raw tile data for text tiles ===");
    let tile_base = ppu.get_bg_tile_base(0) as usize;
    let vram = ppu.vram();

    for tile_num in [473u16, 474, 475, 476, 480, 481] {
        let tile_offset = tile_base + (tile_num as usize) * 32;
        print!("Tile {} (offset 0x{:04X}): ", tile_num, tile_offset);
        if tile_offset + 32 <= vram.len() {
            let mut colors = std::collections::HashSet::new();
            for byte_idx in 0..32 {
                let byte = vram[tile_offset + byte_idx];
                let hi = byte >> 4;
                let lo = byte & 0xF;
                colors.insert(lo);
                colors.insert(hi);
            }
            let has_zero = colors.contains(&0);
            let non_zero_colors: Vec<u8> = colors.iter().filter(|&&c| c != 0).copied().collect();
            println!(
                "color_indices: {:?}, has_zero={}, non_zero_count={}",
                colors.iter().take(10).collect::<Vec<_>>(),
                has_zero,
                non_zero_colors.len()
            );

            if tile_num == 473 {
                println!("  Tile 473 full dump (4bpp, palette 11):");
                for row in 0..8 {
                    let byte = vram[tile_offset + row * 4];
                    let p0 = byte & 0xF;
                    let p1 = (byte >> 4) & 0xF;
                    let byte2 = vram[tile_offset + row * 4 + 1];
                    let p2 = byte2 & 0xF;
                    let p3 = (byte2 >> 4) & 0xF;
                    let byte3 = vram[tile_offset + row * 4 + 2];
                    let p4 = byte3 & 0xF;
                    let p5 = (byte3 >> 4) & 0xF;
                    let byte4 = vram[tile_offset + row * 4 + 3];
                    let p6 = byte4 & 0xF;
                    let p7 = (byte4 >> 4) & 0xF;
                    println!(
                        "    row {}: {} {} {} {} {} {} {} {}",
                        row, p0, p1, p2, p3, p4, p5, p6, p7
                    );
                }
            }
        } else {
            println!("OUT OF BOUNDS");
        }
    }

    println!("\n=== Palette 11 colors ===");
    for i in 0..16u16 {
        let color = gba.get_palette_color(0, 11 * 16 + i);
        let r = color & 0x1F;
        let g = (color >> 5) & 0x1F;
        let b = (color >> 10) & 0x1F;
        println!("  pal11[{}] = 0x{:04X} (r={} g={} b={})", i, color, r, g, b);
    }

    println!("\n=== Direct pixel lookup for text region ===");
    for y in 0..16u16 {
        for x in 0..64u16 {
            if let Some(color) = gba.get_bg_pixel(ppu, mode, 0, x, y) {
                let r = color & 0x1F;
                let g = (color >> 5) & 0x1F;
                let b = (color >> 10) & 0x1F;
                if color != 0x7E80 {
                    println!(
                        "  DIFFERENT at ({}, {}): 0x{:04X} (r={} g={} b={})",
                        x, y, color, r, g, b
                    );
                }
            }
        }
    }

    println!("\n=== BG0 pixel dump row 0 (first 64 pixels) ===");
    let mut all_same = true;
    for x in 0..64u16 {
        if let Some(color) = gba.get_bg_pixel(ppu, mode, 0, x, 0) {
            if color != 0x7E80 {
                all_same = false;
            }
        }
    }
    println!("All same (0x7E80): {}", all_same);

    if all_same {
        let bg_x = ((0u32 + hofs as u32) % 256) as u16;
        let (mx, my) = ppu.apply_bg_mosaic(bg_x, 0);
        println!("Pixel (0,0) -> bg({}, 0) -> mosaic({}, 0)", bg_x, mx);
        let tile_x = mx / 8;
        let tile_y = 0 / 8;
        let pixel_x = mx % 8;
        let pixel_y = 0 % 8;
        println!(
            "  -> tile({}, {}) pixel({}, {}) = tile_num_at_map({}, {})",
            tile_x, tile_y, pixel_x, pixel_y, tile_x, tile_y
        );

        let screen_base = ppu.get_bg_map_base(0) as usize;
        let entry = ppu.get_screen_entry(screen_base, tile_x, tile_y, 0, 32, 32);
        let (tile_num, flip_h, flip_v, palette_num, _) = rgba::Ppu::parse_screen_entry(entry);
        println!(
            "  -> screen entry: tile={}, pal={}, fh={}, fv={}",
            tile_num, palette_num, flip_h, flip_v
        );

        let tile_offset = tile_base + (tile_num as usize) * 32;
        let color_index = ppu.get_tile_pixel_4bpp(
            tile_base,
            tile_num,
            pixel_x as u8,
            pixel_y as u8,
            palette_num,
            false,
            false,
        );
        println!("  -> color_index from tile: {}", color_index);

        let color = gba.get_palette_color(0, (palette_num as u16) * 16 + color_index as u16);
        println!("  -> palette[11][{}] = 0x{:04X}", color_index, color);
    }
}
