use rgba::Gba;
use rgba::KeyState;
use rgba::Ppu;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }
    for round in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();

    let x = 120u16;
    let y = 80u16;

    println!("=== Tracing pixel ({}, {}) ===", x, y);
    println!("DISPCNT: {:#06X}", ppu.get_dispcnt());
    println!("Mode: {}", ppu.get_display_mode());

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let enabled = ppu.is_bg_enabled(bg);
        let priority = ppu.get_bg_priority(bg);
        let is_8bpp = (bgcnt & 0x80) != 0;
        let tile_base = ppu.get_bg_tile_base(bg) as usize;
        let map_base = ppu.get_bg_map_base(bg) as usize;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let bg_size = (bgcnt >> 14) & 0x3;
        let width = 256u16;
        let height = 256u16;

        println!("\nBG{}: enabled={} priority={} 8bpp={} tile_base={:#X} map_base={:#X} hofs={} vofs={} bgcnt={:#06X}",
            bg, enabled, priority, is_8bpp, tile_base, map_base, hofs, vofs, bgcnt);

        if !enabled {
            continue;
        }

        let bg_x = ((x as u32 + hofs as u32) % width as u32) as u16;
        let bg_y = ((y as u32 + vofs as u32) % height as u32) as u16;
        println!("  bg_x={} bg_y={}", bg_x, bg_y);

        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;
        println!(
            "  tile=({}, {}) pixel=({}, {})",
            tile_x, tile_y, pixel_x, pixel_y
        );

        let entry = ppu.get_screen_entry(map_base, tile_x, tile_y, bg_size, width / 8, height / 8);
        let (tile_num, flip_h, flip_v, palette_num, _) = Ppu::parse_screen_entry(entry);
        println!(
            "  entry={:#06X} tile_num={} flip_h={} flip_v={} palette_num={}",
            entry, tile_num, flip_h, flip_v, palette_num
        );

        if is_8bpp {
            let tile_offset = tile_base + (tile_num as usize * 64);
            println!(
                "  8bpp tile_offset={:#X} (tile_base={:#X} + {}*64)",
                tile_offset, tile_base, tile_num
            );
            if tile_offset + 64 > vram.len() {
                println!("  *** OUT OF BOUNDS! vram.len={:#X}", vram.len());
            } else {
                let px = if flip_h { 7 - pixel_x } else { pixel_x };
                let py = if flip_v { 7 - pixel_y } else { pixel_y };
                let pixel_offset = tile_offset + (py as usize * 8) + (px as usize);
                let color_index = vram[pixel_offset];
                println!(
                    "  pixel_offset={:#X} color_index={}",
                    pixel_offset, color_index
                );

                let pal_offset = (color_index as usize) * 2;
                let palette = gba.mem().palette();
                if pal_offset + 1 < palette.len() {
                    let pal_color =
                        u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]]);
                    println!(
                        "  pal_offset={:#X} palette[{}]={:#06X}",
                        pal_offset, color_index, pal_color
                    );
                }
            }
        } else {
            let tile_offset = tile_base + (tile_num as usize * 32);
            println!("  4bpp tile_offset={:#X}", tile_offset);
            if tile_offset + 32 <= vram.len() {
                let px = if flip_h { 7 - pixel_x } else { pixel_x };
                let py = if flip_v { 7 - pixel_y } else { pixel_y };
                let row_offset = tile_offset + (py as usize * 4);
                let byte_val = vram[row_offset + (px as usize / 2)];
                let color_index = if px % 2 == 0 {
                    byte_val & 0x0F
                } else {
                    byte_val >> 4
                };
                println!("  color_index={}", color_index);

                if color_index != 0 {
                    let pal_index = (palette_num as usize * 16) + color_index as usize;
                    let pal_offset = pal_index * 2;
                    let palette = gba.mem().palette();
                    if pal_offset + 1 < palette.len() {
                        let pal_color =
                            u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]]);
                        println!(
                            "  pal_index={} pal_offset={:#X} palette_color={:#06X}",
                            pal_index, pal_offset, pal_color
                        );
                    }
                }
            }
        }

        let result = gba.get_bg_pixel(ppu, ppu.get_display_mode(), bg, x, y);
        println!("  get_bg_pixel() => {:?}", result);
    }

    println!("\n=== Final pixel color ===");
    let final_color = gba.get_pixel_tile_mode(x, y);
    println!("get_pixel_tile_mode({}, {}) = {:#06X}", x, y, final_color);
    let r = (final_color & 0x1F) as u32;
    let g = ((final_color >> 5) & 0x1F) as u32;
    let b = ((final_color >> 10) & 0x1F) as u32;
    println!("RGB555: ({}, {}, {})", r, g, b);

    println!("\n=== Palette RAM (first 32 entries) ===");
    let palette = gba.mem().palette();
    for i in 0..32 {
        let offset = i * 2;
        let color = u16::from_le_bytes([palette[offset], palette[offset + 1]]);
        print!("  [{:3}]={:#06X}", i, color);
        if i % 8 == 7 {
            println!();
        }
    }
    println!();

    println!("\n=== Window visibility at ({}, {}) ===", x, y);
    let win_vis = ppu.get_window_visibility(x, y);
    println!(
        "win_vis = {:#06X} (bits: BG0={} BG1={} BG2={} BG3={} OBJ={})",
        win_vis,
        (win_vis & 1) != 0,
        (win_vis >> 1) & 1,
        (win_vis >> 2) & 1,
        (win_vis >> 3) & 1,
        (win_vis >> 4) & 1,
    );

    println!("\n=== Blend control ===");
    let bldcnt = ppu.get_blend_control();
    println!("BLDCNT={:#06X} mode={}", bldcnt, ppu.get_blend_mode());

    println!("\n=== Window registers ===");
    let io = gba.mem().io();
    let win0h = u16::from_le_bytes([io[0x40], io[0x41]]);
    let win0v = u16::from_le_bytes([io[0x42], io[0x43]]);
    let win1h = u16::from_le_bytes([io[0x44], io[0x45]]);
    let win1v = u16::from_le_bytes([io[0x46], io[0x47]]);
    let winin = u16::from_le_bytes([io[0x48], io[0x49]]);
    let winout = u16::from_le_bytes([io[0x4A], io[0x4B]]);
    println!(
        "WIN0_H={:#06X}  left={} right={}",
        win0h,
        win0h & 0xFF,
        (win0h >> 8) & 0xFF
    );
    println!(
        "WIN0_V={:#06X}  top={} bottom={}",
        win0v,
        win0v & 0xFF,
        (win0v >> 8) & 0xFF
    );
    println!(
        "WIN1_H={:#06X}  left={} right={}",
        win1h,
        win1h & 0xFF,
        (win1h >> 8) & 0xFF
    );
    println!(
        "WIN1_V={:#06X}  top={} bottom={}",
        win1v,
        win1v & 0xFF,
        (win1v >> 8) & 0xFF
    );
    println!(
        "WININ={:#06X}  win0_ctrl={:#05X} win1_ctrl={:#05X}",
        winin,
        winin & 0x1F,
        (winin >> 8) & 0x1F
    );
    println!(
        "WINOUT={:#06X}  outside={:#05X} obj_win={:#05X}",
        winout,
        winout & 0x1F,
        (winout >> 8) & 0x1F
    );

    let dispcnt = ppu.get_dispcnt();
    let win0_en = (dispcnt & (1 << 13)) != 0;
    let win1_en = (dispcnt & (1 << 14)) != 0;
    let obj_win_en = (dispcnt & (1 << 15)) != 0;
    println!(
        "DISPCNT win0_en={} win1_en={} obj_win_en={}",
        win0_en, win1_en, obj_win_en
    );

    let win0_left = win0h & 0xFF;
    let win0_right = (win0h >> 8) & 0xFF;
    let win0_top = win0v & 0xFF;
    let win0_bottom = (win0v >> 8) & 0xFF;
    let in_win0 = 120 >= win0_left && 120 < win0_right && 80 >= win0_top && 80 < win0_bottom;
    println!(
        "Is (120,80) inside WIN0? {} (left={} right={} top={} bottom={})",
        in_win0, win0_left, win0_right, win0_top, win0_bottom
    );

    let win1_left = win1h & 0xFF;
    let win1_right = (win1h >> 8) & 0xFF;
    let win1_top = win1v & 0xFF;
    let win1_bottom = (win1v >> 8) & 0xFF;
    let in_win1 = 120 >= win1_left && 120 < win1_right && 80 >= win1_top && 80 < win1_bottom;
    println!(
        "Is (120,80) inside WIN1? {} (left={} right={} top={} bottom={})",
        in_win1, win1_left, win1_right, win1_top, win1_bottom
    );
}
