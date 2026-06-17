use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    for _ in 0..500 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let ppu = gba.ppu();
    let vram = gba.mem().vram();

    // Scan BG3 (priority 0, highest) for non-transparent pixels
    let bg = 3;
    let bgcnt = ppu.get_bgcnt(bg);
    let char_base = ppu.get_bg_tile_base(bg) as usize;
    let screen_base = ppu.get_bg_map_base(bg) as usize;
    println!(
        "BG3: bgcnt={:#06X} char_base={:#06X} screen_base={:#06X}",
        bgcnt, char_base, screen_base
    );

    let mut non_transparent = 0;
    let mut total = 0;
    let mut sample_pixels = Vec::new();

    for y in 0..160u16 {
        for x in 0..240u16 {
            total += 1;
            if let Some(c) = gba.get_bg_pixel(ppu, 0, bg, x, y) {
                non_transparent += 1;
                if sample_pixels.len() < 10 {
                    sample_pixels.push((x, y, c));
                }
            }
        }
    }

    println!(
        "BG3: {} / {} non-transparent pixels ({:.1}%)",
        non_transparent,
        total,
        non_transparent as f64 / total as f64 * 100.0
    );

    for (x, y, c) in &sample_pixels {
        let r = (c & 0x1F) as u32 * 255 / 31;
        let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
        let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
        println!("  ({},{}) = {:#06X} rgb=({},{},{})", x, y, c, r, g, b);
    }

    // Check each BG layer
    for bg in 0..4 {
        let mut count = 0;
        for y in 0..8u16 {
            for x in 0..8u16 {
                if gba.get_bg_pixel(ppu, 0, bg, x, y).is_some() {
                    count += 1;
                }
            }
        }
        println!("BG{}: {} / 64 non-transparent in (0-7,0-7)", bg, count);
    }
}
