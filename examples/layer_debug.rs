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

    println!("=== BG1 at text region (y=0-15) ===");
    for y in 0..16u16 {
        let mut has_bg1 = false;
        for x in 0..240u16 {
            if gba.get_bg_pixel(ppu, mode, 1, x, y).is_some() {
                has_bg1 = true;
                break;
            }
        }
        if has_bg1 {
            println!("  Row {}: BG1 has opaque pixels", y);
        } else {
            println!("  Row {}: BG1 fully transparent", y);
        }
    }

    println!("\n=== BG2 at text region (y=0-15) ===");
    for y in 0..16u16 {
        let mut has_bg2 = false;
        for x in 0..240u16 {
            if gba.get_bg_pixel(ppu, mode, 2, x, y).is_some() {
                has_bg2 = true;
                break;
            }
        }
        if has_bg2 {
            println!("  Row {}: BG2 has opaque pixels", y);
        } else {
            println!("  Row {}: BG2 fully transparent", y);
        }
    }

    println!("\n=== Rendering comparison ===");
    for y in [0u16, 7, 15] {
        for x in [0u16, 30, 40, 50, 60] {
            let bg0_color = gba.get_bg_pixel(ppu, mode, 0, x, y).unwrap_or(0);
            let final_color = gba.get_pixel_tile_mode(x, y);
            let fb_color = fb[y as usize * 240 + x as usize];

            let fb_r = (fb_color >> 16) & 0xFF;
            let fb_g = (fb_color >> 8) & 0xFF;
            let fb_b = fb_color & 0xFF;

            println!(
                "  ({:2},{:2}): BG0=0x{:04X} final=0x{:04X} FB=({},{},{})",
                x, y, bg0_color, final_color, fb_r, fb_g, fb_b
            );
        }
    }

    println!("\n=== BG1 first non-transparent tile map entries ===");
    let bg1_base = ppu.get_bg_map_base(1) as usize;
    let bg1cnt = ppu.get_bgcnt(1);
    let bg1_size = (bg1cnt >> 14) & 0x3;
    let bg1_w = match bg1_size {
        0 | 2 => 32u16,
        1 | 3 => 64,
        _ => 32,
    };
    let bg1_h = match bg1_size {
        0 | 1 => 32u16,
        2 | 3 => 64,
        _ => 32,
    };
    let mut bg1_count = 0;
    for ty in 0..bg1_h {
        for tx in 0..bg1_w {
            let entry = ppu.get_screen_entry(bg1_base, tx, ty, bg1_size, bg1_w, bg1_h);
            let (tile_num, _, _, pal, _) = rgba::Ppu::parse_screen_entry(entry);
            if tile_num != 0x3FF && tile_num != 0 {
                bg1_count += 1;
                if bg1_count <= 5 {
                    println!("  BG1 map[{},{}] = tile {} pal {}", tx, ty, tile_num, pal);
                }
            }
        }
    }
    println!("BG1 non-zero/non-3FF tiles: {}", bg1_count);
}
