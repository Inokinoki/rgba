use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..1000 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    // Manually render each BG layer separately
    let vram = gba.ppu().vram();
    let pal = gba.mem().palette();
    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);

    for bg in 0..4 {
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if enabled == 0 {
            continue;
        }

        let bgcnt_off = (0x08 + bg * 2) as usize;
        let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
        let char_base = ((bgcnt >> 2) & 3) as usize * 0x4000;
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let priority = bgcnt & 3;

        // Count how many pixels would be non-zero for this BG
        let mut pixel_count = 0;
        let mut color_count = std::collections::HashMap::new();

        for y in 0..160u16 {
            for x in 0..240u16 {
                let map_x = x / 8;
                let map_y = y / 8;
                let entry_idx = map_y as usize * 64 + map_x as usize;
                let entry_off = screen_base + entry_idx * 2;

                if entry_off + 1 >= vram.len() {
                    continue;
                }
                let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
                let tile_num = (entry & 0x3FF) as usize;
                let pal_bank = ((entry >> 12) & 0xF) as usize;

                let tile_x = (x % 8) as usize;
                let tile_y = (y % 8) as usize;
                let tile_off = char_base + tile_num * 32 + tile_y * 4;

                if tile_off >= vram.len() {
                    continue;
                }
                let byte = vram[tile_off + tile_x / 2];
                let color_idx = if tile_x % 2 == 0 {
                    byte & 0xF
                } else {
                    (byte >> 4) & 0xF
                };

                if color_idx != 0 {
                    pixel_count += 1;
                    let pal_idx = pal_bank * 16 + color_idx as usize;
                    let c = if pal_idx * 2 + 1 < pal.len() {
                        u16::from_le_bytes([pal[pal_idx * 2], pal[pal_idx * 2 + 1]])
                    } else {
                        0
                    };
                    *color_count.entry(c).or_insert(0) += 1;
                }
            }
        }

        println!(
            "BG{}: pri={} char={:#06X} screen={:#06X} pixels={}/38400",
            bg, priority, char_base, screen_base, pixel_count
        );

        let mut colors: Vec<_> = color_count.into_iter().collect();
        colors.sort_by(|a, b| b.1.cmp(&a.1));
        for (c, count) in colors.iter().take(5) {
            let r = (c & 0x1F) as u32 * 255 / 31;
            let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
            let b = ((c >> 10) as u32) * 255 / 31;
            println!(
                "  color {:#06X} rgb=({},{},{}): {} pixels",
                c, r, g, b, count
            );
        }
    }

    // Now check: which layer has highest priority with data?
    // BG3 has priority 0 (highest) — does it have data?
    // BG0 has priority 3 (lowest) — it's empty anyway
    println!("\n=== Layer compositing order ===");
    println!("BG3 (pri 0, highest): renders FIRST, shows ON TOP");
    println!("BG2 (pri 1): renders behind BG3");
    println!("BG1 (pri 2): renders behind BG2");
    println!("BG0 (pri 3, lowest): renders behind everything");

    // Actually wait: priority 0 = rendered FIRST = appears on TOP
    // But in get_pixel_tile_mode, it checks bg 0,1,2,3 in order
    // and keeps the one with lower priority number
    // Let me re-read the logic...

    // The code iterates bg 0..4 and keeps the pixel if priority < first_priority
    // So it picks the LOWEST priority (most important) pixel
    // BG3 pri=0 → most important, always wins
    // BG2 pri=1 → beats BG1 and BG0
    // BG1 pri=2 → beats BG0
    // BG0 pri=3 → least important

    // BG3 has tiles with data → it renders
    // But BG3 tiles are mostly tile 1023 (white) → white screen!
    println!("\nBG3 uses tiles that are mostly 1023 (white=0xFF data)");
    println!("This means BG3 paints WHITE over everything!");
}
