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
    let win_vis = ppu.get_window_visibility(3, 3);

    println!(
        "mode={} dispcnt=0x{:04X} win_vis=0x{:04X}",
        mode, dispcnt, win_vis
    );

    let mut first_color = 0u16;
    let mut first_type = 0u8;
    let mut first_priority = 5u8;
    let mut second_color = 0u16;

    for bg in 0..4 {
        if ppu.is_bg_enabled(bg) && (win_vis & (1 << bg)) != 0 {
            let priority = ppu.get_bg_priority(bg) as u8;
            if priority >= first_priority {
                continue;
            }
            if let Some(color) = gba.get_bg_pixel(ppu, mode, bg, 3, 3) {
                second_color = first_color;
                first_color = color;
                first_type = 1;
                first_priority = priority;
            }
        }
    }

    if dispcnt & (1 << 12) != 0 && (win_vis & (1 << 4)) != 0 {
        if let Some((color, priority)) = gba.get_sprite_pixel(ppu, 3, 3) {
            if priority <= first_priority {
                second_color = first_color;
                first_color = color;
                first_type = 2;
                first_priority = priority;
            }
        }
    }

    println!(
        "first_color=0x{:04X} second=0x{:04X} type={} pri={}",
        first_color, second_color, first_type, first_priority
    );

    let bldcnt = ppu.get_blend_control();
    println!("BLDCNT: 0x{:04X}", bldcnt);

    let backdrop = gba.get_palette_color(0, 0);
    println!("Backdrop: 0x{:04X}", backdrop);

    // Now call get_pixel_tile_mode directly
    let result = gba.get_pixel_tile_mode(3, 3);
    println!("get_pixel_tile_mode(3,3): 0x{:04X}", result);

    // The difference: 0x7E80 vs 0x7F99
    // 0x7E80 = 0b0111_1110_1000_0000
    // 0x7F99 = 0b0111_1111_1001_1001
    // These are very different! Something is overwriting the result

    // Check if apply_pixel_blending changes it
    // BLDCNT top 2 bits = special effect: 0=none, 1=alpha blend, 2=...
    let effect = (bldcnt >> 6) & 3;
    println!("Blend effect: {}", effect);
    println!(
        "BLDCNT targets: bg0={} bg1={} bg2={} bg3={} obj={} bd={}",
        (bldcnt >> 0) & 1,
        (bldcnt >> 1) & 1,
        (bldcnt >> 2) & 1,
        (bldcnt >> 3) & 1,
        (bldcnt >> 4) & 1,
        (bldcnt >> 5) & 1
    );
    println!(
        "BLDCNT targets2: bg0={} bg1={} bg2={} bg3={} obj={} bd={}",
        (bldcnt >> 8) & 1,
        (bldcnt >> 9) & 1,
        (bldcnt >> 10) & 1,
        (bldcnt >> 11) & 1,
        (bldcnt >> 12) & 1,
        (bldcnt >> 13) & 1
    );
}
