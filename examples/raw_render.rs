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

    let mut ppm_raw = String::from("P3\n240 160\n255\n");
    for y in 0..160u16 {
        for x in 0..240u16 {
            let mut color = ppu.get_palette_color(0, 0);
            let mut first_priority = 5u8;

            for bg in 0..4 {
                if ppu.is_bg_enabled(bg) {
                    let priority = ppu.get_bg_priority(bg) as u8;
                    if priority >= first_priority {
                        continue;
                    }
                    if let Some(c) = gba.get_bg_pixel(ppu, mode, bg, x, y) {
                        color = c;
                        first_priority = priority;
                    }
                }
            }

            if (ppu.get_dispcnt() & (1 << 12)) != 0 {
                if let Some((c, prio)) = gba.get_sprite_pixel(ppu, x, y) {
                    if prio <= first_priority {
                        color = c;
                    }
                }
            }

            let r = ((color & 0x1F) as u32 * 255 / 31) << 16;
            let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
            let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
            let fb_val = r | g | b;
            ppm_raw.push_str(&format!(
                "{} {} {} ",
                (fb_val >> 16) & 0xFF,
                (fb_val >> 8) & 0xFF,
                fb_val & 0xFF
            ));

            if x < 64 && y < 16 {
                let fb_r = (fb_val >> 16) & 0xFF;
                let fb_g = (fb_val >> 8) & 0xFF;
                let fb_b = fb_val & 0xFF;
                let fb_blend = fb[y as usize * 240 + x as usize];
                let bl_r = (fb_blend >> 16) & 0xFF;
                let bl_g = (fb_blend >> 8) & 0xFF;
                let bl_b = fb_blend & 0xFF;
                if fb_r != bl_r || fb_g != bl_g || fb_b != bl_b {
                    println!(
                        "({:2},{:2}): RAW=({},{},{}) BLEND=({},{},{})",
                        x, y, fb_r, fb_g, fb_b, bl_r, bl_g, bl_b
                    );
                }
            }
        }
        ppm_raw.push('\n');
    }
    std::fs::write("/tmp/frame200_raw.ppm", ppm_raw).unwrap();
    println!("\nSaved /tmp/frame200_raw.ppm (no blending)");
}
