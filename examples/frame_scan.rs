use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..600u32 {
        gba.run_frame_parallel(&mut framebuffer);

        if frame % 60 != 0 && frame < 580 {
            continue;
        }

        gba.sync_ppu_full();
        let dispcnt = gba.ppu().get_dispcnt();
        if dispcnt & 0x80 != 0 {
            println!("Frame {}: FORCED BLANK", frame);
            continue;
        }

        let mut unique = std::collections::HashSet::new();
        for &p in &framebuffer {
            unique.insert(p);
        }

        let enabled_bits = (dispcnt >> 8) & 0xF;
        let mode = dispcnt & 0x7;

        let mut green_count = 0u32;
        for &p in &framebuffer {
            if p == 0x0000FF00 {
                green_count += 1;
            }
        }

        if enabled_bits == 0 {
            println!("Frame {}: no BGs enabled", frame);
            continue;
        }

        println!(
            "Frame {}: dispcnt={:#06X} mode={} colors={} green={:.1}% bg_en={:#05b}",
            frame,
            dispcnt,
            mode,
            unique.len(),
            green_count as f64 / 38400.0 * 100.0,
            enabled_bits
        );

        for bg in 0..4usize {
            if !gba.ppu().is_bg_enabled(bg) {
                continue;
            }
            let bgcnt = gba.ppu().get_bgcnt(bg);
            let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
            let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;

            let vram = gba.ppu().vram();
            let hofs = gba.ppu().get_bg_hofs(bg);
            let vofs = gba.ppu().get_bg_vofs(bg);

            let mut non_empty = 0u32;
            let mut total_checked = 0u32;
            for ty in 0..20u16 {
                for tx in 0..30u16 {
                    let bg_x = tx * 8 + hofs;
                    let bg_y = ty * 8 + vofs;
                    let tile_x = bg_x / 8;
                    let tile_y = bg_y / 8;
                    let entry_off = map_base + (tile_y as usize * 32 + tile_x as usize) * 2;
                    if entry_off + 1 >= vram.len() {
                        continue;
                    }
                    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
                    let tile_num = entry & 0x3FF;
                    total_checked += 1;
                    if tile_num == 0 || tile_num == 1023 {
                        continue;
                    }
                    let tile_off = tile_base + tile_num as usize * 32;
                    if tile_off + 32 > vram.len() {
                        continue;
                    }
                    let has_data = (0..32).any(|i| vram[tile_off + i] != 0);
                    if has_data {
                        non_empty += 1;
                    }
                }
            }
            println!(
                "  BG{}: tile_base={:#X} map={:#X} hofs={} vofs={} non_empty={}/{}",
                bg, tile_base, map_base, hofs, vofs, non_empty, total_checked
            );
        }
    }
}
