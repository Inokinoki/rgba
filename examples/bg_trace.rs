use rgba::Gba;
use rgba::KeyState;

fn save_png(framebuffer: &[u32], path: &str) {
    let width = 240u32;
    let height = 160u32;
    let row_size = ((width * 4 + 3) & !3) as usize;
    let mut bmp = vec![0u8; 54 + row_size * height as usize];
    let row_size = (width * 4 + 3) & !3;
    let file_size = 54 + row_size * height;
    bmp[0..2].copy_from_slice(b"BM");
    bmp[2..6].copy_from_slice(&file_size.to_le_bytes());
    bmp[10..14].copy_from_slice(&54u32.to_le_bytes());
    bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
    bmp[18..22].copy_from_slice(&width.to_le_bytes());
    bmp[22..26].copy_from_slice(&height.to_le_bytes());
    bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
    bmp[28..30].copy_from_slice(&32u16.to_le_bytes());
    for y in 0..height {
        for x in 0..width {
            let src_idx = ((height - 1 - y) * width + x) as usize;
            let dst_idx = (54 + y * row_size + x * 4) as usize;
            let pixel = framebuffer[src_idx];
            bmp[dst_idx] = (pixel & 0xFF) as u8;
            bmp[dst_idx + 1] = ((pixel >> 8) & 0xFF) as u8;
            bmp[dst_idx + 2] = ((pixel >> 16) & 0xFF) as u8;
            bmp[dst_idx + 3] = 0;
        }
    }
    std::fs::write(path, &bmp).unwrap();
}

fn color_to_rgb555(pixel: u32) -> u16 {
    let r = ((pixel >> 16) & 0xFF) as u16 * 31 / 255;
    let g = ((pixel >> 8) & 0xFF) as u16 * 31 / 255;
    let b = (pixel & 0xFF) as u16 * 31 / 255;
    r | (g << 5) | (b << 10)
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    for _ in 0..40 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    save_png(&framebuffer, "/tmp/bg_trace_composited.bmp");
    println!("Saved composited frame");

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let mem = gba.mem();
    let vram = ppu.vram();
    let palette = mem.palette();
    let dispcnt = ppu.get_dispcnt();
    let mode = (dispcnt & 0x7) as u8;

    println!("\n=== Register Dump ===");
    println!("DISPCNT: {:#06X} (mode={})", dispcnt, mode);
    println!(
        "  BG0={} BG1={} BG2={} BG3={} OBJ={} WIN0={} WIN1={} OBJWIN={}",
        (dispcnt >> 8) & 1,
        (dispcnt >> 9) & 1,
        (dispcnt >> 10) & 1,
        (dispcnt >> 11) & 1,
        (dispcnt >> 12) & 1,
        (dispcnt >> 13) & 1,
        (dispcnt >> 14) & 1,
        (dispcnt >> 15) & 1
    );

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let priority = bgcnt & 0x3;
        let tile_base = ((bgcnt >> 2) & 0x3) * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let bg_size = (bgcnt >> 14) & 0x3;
        let is_8bpp = (bgcnt & 0x80) != 0;
        let enabled = ppu.is_bg_enabled(bg);
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        println!("BG{}: enabled={} bgcnt={:#06X} prio={} tile_base={:#06X} map_base={:#06X} size={} 8bpp={} hofs={} vofs={}",
            bg, enabled, bgcnt, priority, tile_base, map_base, bg_size, is_8bpp, hofs, vofs);
    }

    let win0h = ppu.get_bg_hofs(0);
    let io = mem.io();
    let win0h_reg = u16::from_le_bytes([io[0x40], io[0x41]]);
    let win0v_reg = u16::from_le_bytes([io[0x42], io[0x43]]);
    let win1h_reg = u16::from_le_bytes([io[0x44], io[0x45]]);
    let win1v_reg = u16::from_le_bytes([io[0x46], io[0x47]]);
    let winin_reg = u16::from_le_bytes([io[0x48], io[0x49]]);
    let winout_reg = u16::from_le_bytes([io[0x4A], io[0x4B]]);
    println!("\nWindow Registers:");
    println!(
        "WIN0H: {:#06X} (left={} right={})",
        win0h_reg,
        win0h_reg & 0xFF,
        (win0h_reg >> 8) & 0xFF
    );
    println!(
        "WIN0V: {:#06X} (top={} bottom={})",
        win0v_reg,
        win0v_reg & 0xFF,
        (win0v_reg >> 8) & 0xFF
    );
    println!("WIN1H: {:#06X}", win1h_reg);
    println!("WIN1V: {:#06X}", win1v_reg);
    println!(
        "WININ: {:#06X} (win0={:#05X} win1={:#05X})",
        winin_reg,
        winin_reg & 0x3F,
        (winin_reg >> 8) & 0x3F
    );
    println!(
        "WINOUT: {:#06X} (outside={:#05X} objwin={:#05X})",
        winout_reg,
        winout_reg & 0x3F,
        (winout_reg >> 8) & 0x3F
    );

    let bldcnt = ppu.get_blend_control();
    println!("\nBLDCNT: {:#06X} mode={}", bldcnt, (bldcnt >> 6) & 3);
    println!(
        "  1st targets: BG0={} BG1={} BG2={} BG3={} OBJ={}",
        bldcnt & 1,
        (bldcnt >> 1) & 1,
        (bldcnt >> 2) & 1,
        (bldcnt >> 3) & 1,
        (bldcnt >> 4) & 1
    );
    println!(
        "  2nd targets: BG0={} BG1={} BG2={} BG3={} OBJ={}",
        (bldcnt >> 8) & 1,
        (bldcnt >> 9) & 1,
        (bldcnt >> 10) & 1,
        (bldcnt >> 11) & 1,
        (bldcnt >> 12) & 1
    );

    println!(
        "\nBackdrop (pal 0, color 0): {:#06X}",
        mem.read_palette_color(0, 0)
    );

    let mut unique_colors = std::collections::HashMap::new();
    for i in 0..240 * 160 {
        let c = framebuffer[i];
        *unique_colors.entry(c).or_insert(0u32) += 1;
    }
    println!("\nFrame stats: {} unique colors", unique_colors.len());
    let mut colors: Vec<_> = unique_colors.iter().collect();
    colors.sort_by(|a, b| b.1.cmp(a.1));
    for (i, (color, count)) in colors.iter().take(10).enumerate() {
        let rgb555 = color_to_rgb555(**color);
        let r = (rgb555 & 0x1F) * 255 / 31;
        let g = ((rgb555 >> 5) & 0x1F) * 255 / 31;
        let b = ((rgb555 >> 10) & 0x1F) * 255 / 31;
        println!(
            "  #{}: {:#010X} rgb555={:#06X} RGB=({},{},{}) count={} ({:.1}%)",
            i + 1,
            color,
            rgb555,
            r,
            g,
            b,
            count,
            **count as f64 / 38400.0 * 100.0
        );
    }

    println!("\n=== Pixel Traces ===");
    let trace_points = [
        (120, 80, "center"),
        (10, 10, "top-left"),
        (230, 10, "top-right"),
        (10, 150, "bottom-left"),
        (230, 150, "bottom-right"),
        (120, 20, "top-center"),
        (120, 140, "bottom-center"),
        (50, 50, "mid-left"),
        (190, 50, "mid-right"),
        (50, 130, "lower-left"),
    ];

    for &(x, y, label) in &trace_points {
        let pixel = framebuffer[(y as usize) * 240 + (x as usize)];
        let rgb555 = color_to_rgb555(pixel);
        let win_vis = ppu.get_window_visibility(x, y);
        println!(
            "\n--- Pixel ({}, {}) [{}] = {:#010X} rgb555={:#06X} win_vis={:#05X} ---",
            x, y, label, pixel, rgb555, win_vis
        );

        for bg in 0..4u16 {
            if !ppu.is_bg_enabled(bg as usize) {
                println!("  BG{}: DISABLED", bg);
                continue;
            }
            if (win_vis & (1 << bg)) == 0 {
                println!("  BG{}: hidden by window", bg);
                continue;
            }

            let bgcnt = ppu.get_bgcnt(bg as usize);
            let priority = bgcnt & 0x3;
            let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
            let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
            let bg_size = (bgcnt >> 14) & 0x3;
            let is_8bpp = (bgcnt & 0x80) != 0;
            let hofs = ppu.get_bg_hofs(bg as usize);
            let vofs = ppu.get_bg_vofs(bg as usize);

            let (width, height) = match bg_size {
                0 => (256u16, 256u16),
                1 => (512u16, 256u16),
                2 => (256u16, 512u16),
                3 => (512u16, 512u16),
                _ => (256, 256),
            };

            let bg_x = ((x as u32 + hofs as u32) % width as u32) as u16;
            let bg_y = ((y as u32 + vofs as u32) % height as u32) as u16;
            let tile_x = bg_x / 8;
            let tile_y = bg_y / 8;
            let pixel_x = bg_x % 8;
            let pixel_y = bg_y % 8;

            let num_blocks_x = (width / 8) / 32;
            let block_x = (tile_x / 32) as usize;
            let block_y = (tile_y / 32) as usize;
            let local_x = (tile_x % 32) as usize;
            let local_y = (tile_y % 32) as usize;
            let block_num = block_y * num_blocks_x as usize + block_x;
            let entry_off = map_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;

            let entry = if entry_off + 1 < vram.len() {
                u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]])
            } else {
                0
            };
            let tile_num = entry & 0x3FF;
            let flip_h = (entry & 0x400) != 0;
            let flip_v = (entry & 0x800) != 0;
            let pal_num = (entry >> 12) & 0xF;

            let tile_off = tile_base + tile_num as usize * 32;
            let px = if flip_h { 7 - pixel_x } else { pixel_x };
            let py = if flip_v { 7 - pixel_y } else { pixel_y };
            let row_off = tile_off + py as usize * 4;

            let color_idx = if tile_off + 32 <= vram.len() {
                let byte = vram[row_off + px as usize / 2];
                if px % 2 == 0 {
                    byte & 0x0F
                } else {
                    byte >> 4
                }
            } else {
                0
            };

            let pal_idx = if is_8bpp {
                color_idx as u16
            } else {
                pal_num * 16 + color_idx as u16
            };
            let pal_color = mem.read_palette_color(0, pal_idx);

            let r = (pal_color & 0x1F) as u32 * 255 / 31;
            let g = ((pal_color >> 5) & 0x1F) as u32 * 255 / 31;
            let b = ((pal_color >> 10) & 0x1F) as u32 * 255 / 31;

            println!("  BG{}: prio={} bg=({},{}) tile=({},{}) px=({},{}) entry_off={:#X} entry={:#06X} tile={} fliph={} flipv={} pal={} -> idx={} pal_idx={} pal_color={:#06X} RGB=({},{},{})",
                bg, priority, bg_x, bg_y, tile_x, tile_y, pixel_x, pixel_y,
                entry_off, entry, tile_num, flip_h, flip_v, pal_num,
                color_idx, pal_idx, pal_color, r, g, b);
        }

        let sprite_pixel = gba.get_sprite_pixel(ppu, x, y);
        if let Some((color, prio)) = sprite_pixel {
            let r = (color & 0x1F) as u32 * 255 / 31;
            let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
            let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
            println!(
                "  OBJ: prio={} color={:#06X} RGB=({},{},{})",
                prio, color, r, g, b
            );
        } else {
            println!("  OBJ: none");
        }
    }

    println!("\n=== BG3 Screen Entries (first 3 rows, 30 cols) ===");
    {
        let bg = 3usize;
        let bgcnt = ppu.get_bgcnt(bg);
        let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);

        for ty in 0..3u16 {
            print!("  row {}: ", ty);
            for tx in 0..10u16 {
                let bg_x = tx * 8 + hofs;
                let bg_y = ty * 8 + vofs;
                let tile_x = bg_x / 8;
                let tile_y = bg_y / 8;
                let entry_off = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
                let entry = if entry_off + 1 < vram.len() {
                    u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]])
                } else {
                    0
                };
                let tile_num = entry & 0x3FF;
                let pal = (entry >> 12) & 0xF;
                print!("T{}P{} ", tile_num, pal);
            }
            println!();
        }

        let mut tile_counts = std::collections::HashMap::new();
        for ty in 0..20u16 {
            for tx in 0..30u16 {
                let bg_x = tx * 8 + hofs;
                let bg_y = ty * 8 + vofs;
                let tile_x = bg_x / 8;
                let tile_y = bg_y / 8;
                let entry_off = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
                let entry = if entry_off + 1 < vram.len() {
                    u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]])
                } else {
                    0
                };
                let tile_num = entry & 0x3FF;
                *tile_counts.entry(tile_num).or_insert(0u32) += 1;
            }
        }
        let mut tiles: Vec<_> = tile_counts.iter().collect();
        tiles.sort_by(|a, b| b.1.cmp(a.1));
        println!("\n  BG3 top tile nums:");
        for (tile, count) in tiles.iter().take(10) {
            let tile_off = tile_base + **tile as usize * 32;
            let all_zero = tile_off + 32 <= vram.len() && (0..32).all(|i| vram[tile_off + i] == 0);
            println!("    tile {} count={} all_zero={}", tile, count, all_zero);
        }
    }
}
