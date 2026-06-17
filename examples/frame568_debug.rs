use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..568 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = &gba.ppu;
    let mode = ppu.get_display_mode();
    let dispcnt = ppu.get_dispcnt();

    println!("=== Frame 568 PPU state ===");
    println!("DISPCNT: 0x{:04X}, Mode: {}", dispcnt, mode);
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let enabled = ppu.is_bg_enabled(bg);
        let priority = ppu.get_bg_priority(bg);
        println!(
            "BG{}: enabled={}, pri={}, bgcnt=0x{:04X}",
            bg, enabled, priority, bgcnt
        );
    }
    println!(
        "BLDCNT: 0x{:04X}, mode: {}, BLDY: {}",
        ppu.get_blend_control(),
        ppu.get_blend_mode(),
        ppu.get_blend_brightness()
    );

    let obj_enabled = (dispcnt & (1 << 12)) != 0;
    println!("\nOBJ enabled: {}", obj_enabled);

    let mut obj_count = 0;
    for sprite in 0..128 {
        if ppu.sprite_is_enabled(sprite) && !ppu.sprite_is_window(sprite) {
            obj_count += 1;
            if obj_count <= 5 {
                let (w, h) = ppu.sprite_dimensions(sprite);
                let x = ppu.sprite_x(sprite);
                let y = ppu.sprite_y(sprite);
                let prio = ppu.sprite_priority(sprite);
                let affine = ppu.sprite_is_affine(sprite);
                println!(
                    "  OBJ {}: x={}, y={}, {}x{}, pri={}, affine={}",
                    sprite, x, y, w, h, prio, affine
                );
            }
        }
    }
    println!("Total visible OBJ sprites: {}", obj_count);

    println!("\n=== Frame 568 BG layer analysis ===");
    for bg in 0..4 {
        let mut count = 0;
        for y in 0..160u16 {
            for x in 0..240u16 {
                if gba.get_bg_pixel(ppu, mode, bg, x, y).is_some() {
                    count += 1;
                }
            }
        }
        println!("BG{} non-transparent pixels: {}/{}", bg, count, 240 * 160);
    }

    println!("\n=== Frame 568: check text tiles on BG0 ===");
    let screen_base = ppu.get_bg_map_base(0) as usize;
    let bg0cnt = ppu.get_bgcnt(0);
    let bg0_size = (bg0cnt >> 14) & 0x3;
    let bg0_w = match bg0_size {
        0 | 2 => 32u16,
        1 | 3 => 64,
        _ => 32,
    };
    let bg0_h = match bg0_size {
        0 | 1 => 32u16,
        2 | 3 => 64,
        _ => 32,
    };

    let mut text_tiles = 0;
    for ty in 0..bg0_h {
        for tx in 0..bg0_w {
            let entry = ppu.get_screen_entry(screen_base, tx, ty, bg0_size, bg0_w, bg0_h);
            let (tile_num, _, _, pal, _) = rgba::Ppu::parse_screen_entry(entry);
            if tile_num != 0 && tile_num != 0x3FF {
                text_tiles += 1;
                if text_tiles <= 10 {
                    println!("  BG0 map[{},{}] = tile {} (pal {})", tx, ty, tile_num, pal);
                }
            }
        }
    }
    println!("BG0 non-empty tiles: {}", text_tiles);

    println!("\n=== Frame 568: rendering at various points ===");
    for y in [0u16, 40, 80, 120, 140] {
        for x in [0u16, 60, 120, 180] {
            let fb_color = fb[y as usize * 240 + x as usize];
            let fb_r = (fb_color >> 16) & 0xFF;
            let fb_g = (fb_color >> 8) & 0xFF;
            let fb_b = fb_color & 0xFF;
            let final_color = gba.get_pixel_tile_mode(x, y);
            println!(
                "  ({:3},{:3}): final=0x{:04X} FB=({},{},{})",
                x, y, final_color, fb_r, fb_g, fb_b
            );
        }
    }

    let mut ppm = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm.push('\n');
    }
    std::fs::write("/tmp/frame568_detail.ppm", ppm).unwrap();
    println!("\nSaved /tmp/frame568_detail.ppm");
}
