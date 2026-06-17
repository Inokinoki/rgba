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
    let mode = ppu.get_display_mode();
    let dispcnt = ppu.get_dispcnt();
    let bldcnt = ppu.get_blend_control();
    let bldy = ppu.get_blend_brightness();
    let blend_mode = ppu.get_blend_mode();
    let win_vis_all = 0x1Fu16;

    println!("=== PPU State at Frame 200 ===");
    println!("DISPCNT: 0x{:04X}, Mode: {}", dispcnt, mode);
    println!("BLDCNT: 0x{:04X}, Blend mode: {}", bldcnt, blend_mode);
    println!("BLDY: {}", bldy);

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let enabled = ppu.is_bg_enabled(bg);
        let priority = ppu.get_bg_priority(bg);
        let tile_base = ppu.get_bg_tile_base(bg);
        let map_base = ppu.get_bg_map_base(bg);
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        println!(
            "BG{}: enabled={}, pri={}, bgcnt=0x{:04X}, tile_base=0x{:04X}, map_base=0x{:04X}, hofs={}, vofs={}",
            bg, enabled, priority, bgcnt, tile_base, map_base, hofs, vofs
        );
    }

    println!("\n=== Scanning for BG0 non-transparent pixels ===");
    let mut bg0_count = 0;
    let mut bg0_samples: Vec<(u16, u16, u16)> = Vec::new();
    let mut all_bg0_colors: std::collections::HashSet<u16> = std::collections::HashSet::new();

    for y in 0..160u16 {
        for x in 0..240u16 {
            if let Some(color) = gba.get_bg_pixel(ppu, mode, 0, x, y) {
                bg0_count += 1;
                all_bg0_colors.insert(color);
                if bg0_samples.len() < 20 {
                    bg0_samples.push((x, y, color));
                }
            }
        }
    }

    println!("BG0 non-transparent pixels: {}/{}", bg0_count, 240 * 160);
    println!("Unique BG0 colors: {}", all_bg0_colors.len());
    println!("Sample BG0 pixels (x, y, color):");
    for (x, y, c) in &bg0_samples {
        let r = c & 0x1F;
        let g = (c >> 5) & 0x1F;
        let b = (c >> 10) & 0x1F;
        println!("  ({}, {}) -> 0x{:04X} (r={} g={} b={})", x, y, c, r, g, b);
    }

    println!("\n=== Checking framebuffer at BG0 pixel locations ===");
    for (x, y, bg0_color) in &bg0_samples {
        let fb_color = fb[*y as usize * 240 + *x as usize];
        let fb_r = (fb_color >> 16) & 0xFF;
        let fb_g = (fb_color >> 8) & 0xFF;
        let fb_b = fb_color & 0xFF;
        let r = (bg0_color & 0x1F) as u32 * 255 / 31;
        let g = ((bg0_color >> 5) & 0x1F) as u32 * 255 / 31;
        let b = ((bg0_color >> 10) & 0x1F) as u32 * 255 / 31;
        println!(
            "  ({}, {}) BG0=0x{:04X} (rgb={},{},{}) FB=(rgb={},{},{}) match={}",
            x,
            y,
            bg0_color,
            r,
            g,
            b,
            fb_r,
            fb_g,
            fb_b,
            r == fb_r && g == fb_g && b == fb_b
        );
    }

    println!("\n=== Checking get_pixel_tile_mode at BG0 locations ===");
    for (x, y, bg0_color) in bg0_samples.iter().take(5) {
        let final_color = gba.get_pixel_tile_mode(*x, *y);
        let fr = final_color & 0x1F;
        let fg = (final_color >> 5) & 0x1F;
        let fb_val = (final_color >> 10) & 0x1F;
        println!(
            "  ({}, {}) BG0=0x{:04X} -> final=0x{:04X} (r={} g={} b={})",
            x, y, bg0_color, final_color, fr, fg, fb_val
        );
    }

    println!("\n=== Checking backdrop color ===");
    let backdrop = gba.get_palette_color(0, 0);
    let dr = backdrop & 0x1F;
    let dg = (backdrop >> 5) & 0x1F;
    let db = (backdrop >> 10) & 0x1F;
    println!("Backdrop: 0x{:04X} (r={} g={} b={})", backdrop, dr, dg, db);

    let bg0_color_sample = bg0_samples.first().map(|s| s.2).unwrap_or(0);
    if bg0_color_sample != 0 {
        let blended = blend_brightness_up_manual(bg0_color_sample, bldy as u32);
        println!(
            "blend_up(0x{:04X}, {}) = 0x{:04X}",
            bg0_color_sample, bldy, blended
        );
        let blended_no_bldy = blend_brightness_up_manual(bg0_color_sample, 13);
        println!(
            "blend_up(0x{:04X}, 13) = 0x{:04X}",
            bg0_color_sample, blended_no_bldy
        );
    }

    println!("\n=== Checking BG0 tile map around text region ===");
    for tile_y in 0..4u16 {
        for tile_x in 0..30u16 {
            let screen_base = ppu.get_bg_map_base(0) as usize;
            let entry = ppu.get_screen_entry(screen_base, tile_x, tile_y, 0, 32, 32);
            let (tile_num, flip_h, flip_v, palette_num, _) = rgba::Ppu::parse_screen_entry(entry);
            if tile_num != 0 && tile_num != 0x3FF {
                if tile_y < 3 && tile_x < 32 {
                    println!(
                        "  tile({},{}) = num={}, pal={}, fh={}, fv={}",
                        tile_x, tile_y, tile_num, palette_num, flip_h, flip_v
                    );
                }
            }
        }
    }

    println!("\n=== Check BG1/BG2/BG3 for non-transparent pixels ===");
    for bg in 1..4 {
        let mut count = 0;
        for y in 0..160u16 {
            for x in 0..240u16 {
                if gba.get_bg_pixel(ppu, mode, bg, x, y).is_some() {
                    count += 1;
                }
            }
        }
        println!("BG{} non-transparent pixels: {}", bg, count);
    }

    println!("\n=== Full framebuffer dark pixel scan (r+g+b < 100) ===");
    let mut dark_count = 0;
    for y in 0..160usize {
        for x in 0..240usize {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            if (r as u32) + (g as u32) + (b as u32) < 100 {
                dark_count += 1;
                if dark_count <= 10 {
                    println!("  dark pixel at ({}, {}): rgb=({},{},{})", x, y, r, g, b);
                }
            }
        }
    }
    println!("Total dark pixels: {}", dark_count);
}

fn blend_brightness_up_manual(c: u16, ey: u32) -> u16 {
    let r = ((c & 0x1F) as u32 + ((31 - (c & 0x1F) as u32) * ey) / 16);
    let g = (((c >> 5) & 0x1F) as u32 + ((31 - ((c >> 5) & 0x1F) as u32) * ey) / 16);
    let b = (((c >> 10) & 0x1F) as u32 + ((31 - ((c >> 10) & 0x1F) as u32) * ey) / 16);
    r.min(31) as u16 | ((g.min(31) as u16) << 5) | ((b.min(31) as u16) << 10)
}
