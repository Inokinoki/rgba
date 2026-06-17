use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    for _ in 0..1000 {
        gba.run_frame();
    }
    gba.sync_ppu_full();
    gba.sync_ppu();

    let ppu = gba.ppu();
    let vram = ppu.vram();

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let tile_base = ppu.get_bg_tile_base(bg) as usize;
        let map_base = ppu.get_bg_map_base(bg) as usize;
        let size = (bgcnt >> 14) & 0x3;
        let is_8bpp = (bgcnt & 0x80) != 0;
        let prio = ppu.get_bg_priority(bg);
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);

        eprintln!("\n=== BG{}: bgcnt=0x{:04X} prio={} tile_base=0x{:05X} map_base=0x{:05X} size={} 8bpp={} hofs={} vofs={} ===",
            bg, bgcnt, prio, tile_base, map_base, size, is_8bpp, hofs, vofs);

        // Decode bgcnt manually per GBATEK
        let prio_gba = bgcnt & 0x3;
        let char_base = ((bgcnt >> 2) & 0x3) * 0x4000;
        let mosaic = (bgcnt >> 6) & 1;
        let color_mode = (bgcnt >> 7) & 1;
        let screen_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let overflow = (bgcnt >> 13) & 1;
        let screen_size = (bgcnt >> 14) & 0x3;
        eprintln!("  GBATEK decode: prio={} char_base=0x{:05X} mosaic={} color_mode={} screen_base=0x{:05X} overflow={} size={}",
            prio_gba, char_base, mosaic, color_mode, screen_base, overflow, screen_size);

        // Check screen map: first 64 entries
        eprintln!("  Screen map entries (first 64):");
        let (w, h) = match screen_size {
            0 => (32u16, 32u16),
            1 => (64u16, 32u16),
            2 => (32u16, 64u16),
            3 => (64u16, 64u16),
            _ => (32, 32),
        };
        for row in 0..8 {
            let mut line = String::new();
            for col in 0..16 {
                let block_x = col / 32;
                let block_y = row / 32;
                let local_x = col % 32;
                let local_y = row % 32;
                let num_blocks_x = w / 32;
                let block_num = block_y * (num_blocks_x as usize) + block_x;
                let off = map_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;
                if off + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
                    let tile = entry & 0x3FF;
                    use std::fmt::Write;
                    write!(&mut line, "{:4}", tile).unwrap();
                }
            }
            eprintln!("    row {}: {}", row, line);
        }

        // Check tile data at tile_base: first few tiles
        eprintln!("  Tile data at 0x{:05X} (first 4 tiles):", tile_base);
        for tile in 0..4 {
            let bytes_per_tile = if color_mode != 0 { 64 } else { 32 };
            let off = tile_base + tile * bytes_per_tile;
            let mut all_zero = true;
            let mut all_ff = true;
            if off + bytes_per_tile <= vram.len() {
                for b in 0..bytes_per_tile {
                    if vram[off + b] != 0 {
                        all_zero = false;
                    }
                    if vram[off + b] != 0xFF {
                        all_ff = false;
                    }
                }
            }
            eprintln!(
                "    tile {}: offset=0x{:05X} {}",
                tile,
                off,
                if all_zero {
                    "ALL ZERO"
                } else if all_ff {
                    "ALL 0xFF"
                } else {
                    "HAS DATA"
                }
            );
            if !all_zero && !all_ff && tile < 2 {
                // Print first 8 bytes
                let mut hex = String::new();
                for b in 0..8.min(bytes_per_tile) {
                    use std::fmt::Write;
                    write!(&mut hex, "{:02X} ", vram[off + b]).unwrap();
                }
                eprintln!("      data: {}", hex);
            }
        }
    }

    // Check BG palette
    let pal = gba.mem().palette();
    eprintln!("\nBG Palette (first 32 colors):");
    for i in 0..32 {
        let c = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        if c != 0 {
            let r = c & 0x1F;
            let g = (c >> 5) & 0x1F;
            let b = (c >> 10) & 0x1F;
            if i < 16 || (i >= 16 && i < 32) {
                eprintln!("  pal[{}]: 0x{:04X} (r={} g={} b={})", i, c, r, g, b);
            }
        }
    }

    // Trace a specific pixel (120, 80) through BG3 rendering
    eprintln!("\n=== Tracing pixel (120, 80) through BG3 ===");
    let bg = 3;
    let x = 120u16;
    let y = 80u16;
    let bgcnt = ppu.get_bgcnt(bg);
    let tile_base = ppu.get_bg_tile_base(bg) as usize;
    let map_base = ppu.get_bg_map_base(bg) as usize;
    let size = (bgcnt >> 14) & 0x3;
    let hofs = ppu.get_bg_hofs(bg);
    let vofs = ppu.get_bg_vofs(bg);
    let is_8bpp = (bgcnt & 0x80) != 0;

    let width = match size {
        0 => 256,
        1 => 512,
        2 => 256,
        3 => 512,
        _ => 256,
    };
    let height = match size {
        0 => 256,
        1 => 256,
        2 => 512,
        3 => 512,
        _ => 256,
    };

    let bg_x = ((x as u32 + hofs as u32) % width as u32) as u16;
    let bg_y = ((y as u32 + vofs as u32) % height as u32) as u16;
    let tile_x = bg_x / 8;
    let tile_y = bg_y / 8;
    let pixel_x = bg_x % 8;
    let pixel_y = bg_y % 8;

    eprintln!(
        "  hofs={} vofs={} -> bg_x={} bg_y={}",
        hofs, vofs, bg_x, bg_y
    );
    eprintln!(
        "  tile=({}, {}) pixel=({}, {})",
        tile_x, tile_y, pixel_x, pixel_y
    );

    let entry = ppu.get_screen_entry(map_base, tile_x, tile_y, size, width / 8, height / 8);
    let (tile_num, flip_h, flip_v, palette_num, _) = rgba::Ppu::parse_screen_entry(entry);
    eprintln!(
        "  screen_entry=0x{:04X}: tile={} flip_h={} flip_v={} pal={}",
        entry, tile_num, flip_h, flip_v, palette_num
    );

    let bytes_per_tile = if is_8bpp { 64 } else { 32 };
    let tile_offset = tile_base + tile_num as usize * bytes_per_tile;
    eprintln!(
        "  tile_offset=0x{:05X} (tile_base=0x{:05X} + {} * {})",
        tile_offset, tile_base, tile_num, bytes_per_tile
    );

    if tile_offset + bytes_per_tile <= vram.len() {
        let color_index = if is_8bpp {
            ppu.get_tile_pixel_8bpp(
                tile_base,
                tile_num,
                pixel_x as u8,
                pixel_y as u8,
                flip_h,
                flip_v,
            )
        } else {
            ppu.get_tile_pixel_4bpp(
                tile_base,
                tile_num,
                pixel_x as u8,
                pixel_y as u8,
                palette_num,
                flip_h,
                flip_v,
            )
        };
        eprintln!("  color_index={}", color_index);

        if color_index != 0 {
            let pal_index = if is_8bpp {
                color_index as u16
            } else {
                palette_num * 16 + color_index as u16
            };
            let color = gba.get_palette_color(0, pal_index);
            let r = color & 0x1F;
            let g = (color >> 5) & 0x1F;
            let b = (color >> 10) & 0x1F;
            eprintln!(
                "  pal_index={} color=0x{:04X} (r={} g={} b={})",
                pal_index, color, r, g, b
            );
        } else {
            eprintln!("  transparent (color_index == 0)");
        }
    } else {
        eprintln!("  OUT OF VRAM BOUNDS!");
    }
}
