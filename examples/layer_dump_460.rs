use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..460 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = &gba.ppu;
    let dispcnt = ppu.get_dispcnt();
    println!("DISPCNT: 0x{:04X}", dispcnt);
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let priority = bgcnt & 0x3;
        let tile_base = ((bgcnt >> 2) & 0x3) as usize * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let mosaic = (bgcnt >> 6) & 1;
        let color_mode = (bgcnt >> 7) & 1;
        let size = (bgcnt >> 14) & 0x3;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let enabled = ppu.is_bg_enabled(bg);
        println!(
            "BG{}: enabled={} cnt=0x{:04X} pri={} tile=0x{:04X} map=0x{:04X} mosaic={} {}bpp size={} hofs={} vofs={}",
            bg, enabled, bgcnt, priority, tile_base, map_base, mosaic,
            if color_mode == 0 { "4" } else { "8" },
            size, hofs, vofs
        );
    }

    let bldcnt = ppu.get_blend_control();
    let bldy = ppu.get_blend_brightness();
    println!("BLDCNT: 0x{:04X} BLDY: {}", bldcnt, bldy);

    let mut bg0_only = String::from("P3\n240 160\n255\n");
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = if let Some(color) = gba.get_bg_pixel(ppu, 0, 0, x, y) {
                let r = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
                let b = (color & 0x1F) as u32 * 255 / 31;
                format!("{} {} {} ", r, g, b)
            } else {
                "32 32 32 ".to_string()
            };
            bg0_only.push_str(&c);
        }
        bg0_only.push('\n');
    }
    std::fs::write("/tmp/bg0_only_460.ppm", bg0_only).unwrap();
    println!("Saved BG0-only to /tmp/bg0_only_460.ppm");

    let mut bg1_only = String::from("P3\n240 160\n255\n");
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = if let Some(color) = gba.get_bg_pixel(ppu, 0, 1, x, y) {
                let r = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
                let b = (color & 0x1F) as u32 * 255 / 31;
                format!("{} {} {} ", r, g, b)
            } else {
                "32 32 32 ".to_string()
            };
            bg1_only.push_str(&c);
        }
        bg1_only.push('\n');
    }
    std::fs::write("/tmp/bg1_only_460.ppm", bg1_only).unwrap();
    println!("Saved BG1-only to /tmp/bg1_only_460.ppm");

    let mut bg2_only = String::from("P3\n240 160\n255\n");
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = if let Some(color) = gba.get_bg_pixel(ppu, 0, 2, x, y) {
                let r = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
                let b = (color & 0x1F) as u32 * 255 / 31;
                format!("{} {} {} ", r, g, b)
            } else {
                "32 32 32 ".to_string()
            };
            bg2_only.push_str(&c);
        }
        bg2_only.push('\n');
    }
    std::fs::write("/tmp/bg2_only_460.ppm", bg2_only).unwrap();
    println!("Saved BG2-only to /tmp/bg2_only_460.ppm");

    let mut bg3_only = String::from("P3\n240 160\n255\n");
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = if let Some(color) = gba.get_bg_pixel(ppu, 0, 3, x, y) {
                let r = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
                let b = (color & 0x1F) as u32 * 255 / 31;
                format!("{} {} {} ", r, g, b)
            } else {
                "32 32 32 ".to_string()
            };
            bg3_only.push_str(&c);
        }
        bg3_only.push('\n');
    }
    std::fs::write("/tmp/bg3_only_460.ppm", bg3_only).unwrap();
    println!("Saved BG3-only to /tmp/bg3_only_460.ppm");
}
