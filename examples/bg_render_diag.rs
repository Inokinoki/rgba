use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..600 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = gba.ppu();
    let vram = ppu.vram();
    let pal = gba.mem().palette();
    let dispcnt = ppu.get_dispcnt();

    // Dump all 4 BG layers individually for comparison
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let priority = bgcnt & 3;
        let char_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let bg_size = (bgcnt >> 14) & 3;
        let is_8bpp = (bgcnt & 0x80) != 0;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);

        println!("=== BG{}: bgcnt={:04X} pri={} char={:05X} screen={:05X} {}bpp size={} hofs={} vofs={} ===",
            bg, bgcnt, priority, char_base, screen_base,
            if is_8bpp { "8" } else { "4" }, bg_size, hofs, vofs);

        // Manually render BG layer and save
        let mut layer_fb = vec![0u32; 240 * 160];
        let mut nonzero = 0u32;
        let mut unique_colors = std::collections::HashSet::new();

        for y in 0..160u16 {
            for x in 0..240u16 {
                if let Some(color) = gba.get_bg_pixel(ppu, 0, bg, x, y) {
                    let r = ((color & 0x1F) as u32 * 255 / 31) << 16;
                    let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
                    let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                    layer_fb[y as usize * 240 + x as usize] = r | g | b;
                    nonzero += 1;
                    unique_colors.insert(color);
                }
            }
        }

        println!(
            "  {} nonzero pixels, {} unique colors",
            nonzero,
            unique_colors.len()
        );

        // Save as PPM
        let mut out = Vec::new();
        out.extend_from_slice(b"P6\n240 160\n255\n");
        for y in 0..160usize {
            for x in 0..240usize {
                let p = layer_fb[y * 240 + x];
                out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8]);
            }
        }
        let path = format!("/tmp/bg{}_layer.ppm", bg);
        std::fs::write(&path, &out).unwrap();

        // Also dump what screen entries look like for first visible row
        // For BG with hofs, calculate which tiles are visible
        println!("  Screen entries for visible area (y=0):");
        let (map_width, _) = match bg_size {
            0 => (256, 256),
            1 => (512, 256),
            2 => (256, 512),
            3 => (512, 512),
            _ => (256, 256),
        };
        for x_tile in 0..4 {
            let px = (x_tile as u32 * 8 + hofs as u32) % map_width as u32;
            let py = (0u32 + vofs as u32) % 256;
            let tile_x = px / 8;
            let tile_y = py / 8;

            let tiles_per_row = map_width / 8;
            let num_blocks_x = tiles_per_row / 32;
            let block_x = (tile_x / 32) as usize;
            let block_y = (tile_y / 32) as usize;
            let local_x = (tile_x % 32) as usize;
            let local_y = (tile_y % 32) as usize;
            let block_num = block_y * num_blocks_x + block_x;
            let entry_offset = screen_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;

            let entry = if entry_offset + 1 < vram.len() {
                u16::from_le_bytes([vram[entry_offset], vram[entry_offset + 1]])
            } else {
                0
            };
            let tile_num = entry & 0x3FF;
            let flip_h = (entry & 0x400) != 0;
            let flip_v = (entry & 0x800) != 0;
            let pal_num = (entry >> 12) & 0xF;

            // Show first few bytes of tile data
            let tile_off = char_base + tile_num as usize * 32;
            let mut tile_data = [0u8; 16];
            if tile_off + 16 <= vram.len() {
                tile_data.copy_from_slice(&vram[tile_off..tile_off + 16]);
            }

            println!("    px={:3} tile=({},{}) entry_off={:05X} entry={:04X} tn={} pal={} f{}{} data={:02X}{:02X}{:02X}{:02X}...",
                px, tile_x, tile_y, entry_offset, entry, tile_num, pal_num,
                if flip_h { "H" } else { "" }, if flip_v { "V" } else { "" },
                tile_data[0], tile_data[1], tile_data[2], tile_data[3]);
        }

        // Check palette entries for this BG
        println!("  Palette entries 0-15 of palette 0:");
        for i in 0..16 {
            let off = i * 2;
            let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
            if c != 0 {
                let r = c & 0x1F;
                let g = (c >> 5) & 0x1F;
                let b = (c >> 10) & 0x1F;
                print!("  [{}]={:04X}(R{}G{}B{})", i, c, r, g, b);
            }
        }
        println!();

        // Now check: are the rendered pixels matching what they should be?
        // Pick pixel (120, 80) and trace through the rendering
        let x = 120u16;
        let y = 80u16;
        if let Some(color) = gba.get_bg_pixel(ppu, 0, bg, x, y) {
            let px = ((x as u32 + hofs as u32) % map_width as u32) as u16;
            let py = ((y as u32 + vofs as u32) % 256) as u16;
            let tile_x = px / 8;
            let tile_y = py / 8;
            let pixel_x = px % 8;
            let pixel_y = py % 8;

            let tiles_per_row = map_width / 8;
            let num_blocks_x = tiles_per_row / 32;
            let block_x = (tile_x / 32) as usize;
            let block_y = (tile_y / 32) as usize;
            let local_x = (tile_x % 32) as usize;
            let local_y = (tile_y % 32) as usize;
            let block_num = block_y * num_blocks_x + block_x;
            let entry_offset = screen_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;

            let entry = if entry_offset + 1 < vram.len() {
                u16::from_le_bytes([vram[entry_offset], vram[entry_offset + 1]])
            } else {
                0
            };
            let tile_num = entry & 0x3FF;
            let pal_num = (entry >> 12) & 0xF;

            let tile_off = char_base + tile_num as usize * 32;
            let row_off = tile_off + (pixel_y as usize * 4);
            let byte = if row_off < vram.len() {
                vram[row_off + (pixel_x as usize / 2)]
            } else {
                0
            };
            let nibble = if pixel_x % 2 == 0 {
                byte & 0xF
            } else {
                byte >> 4
            };
            let pal_idx = pal_num as usize * 16 + nibble as usize;
            let expected_color = u16::from_le_bytes([pal[pal_idx * 2], pal[pal_idx * 2 + 1]]);

            println!("  Pixel ({},{}): px=({}, {}) tile=({},{}) pix=({},{}) entry={:04X} tn={} nibble={} pal_idx={} expected={:04X} got={:04X} {}",
                x, y, px, py, tile_x, tile_y, pixel_x, pixel_y, entry, tile_num, nibble, pal_idx,
                expected_color, color, if expected_color == color { "OK" } else { "MISMATCH!" });
        } else {
            println!("  Pixel ({},{}): transparent", x, y);
        }
    }

    println!("\nAll BG layers saved to /tmp/bg[0-3]_layer.ppm");
}
