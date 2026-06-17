use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    let mut fb_noblend = vec![0u32; 240 * 160];

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = &gba.ppu;

    let bldcnt_orig = ppu.get_blend_control();
    let bldy_orig = ppu.get_blend_brightness();

    let forced_blank = ppu.get_dispcnt() & 0x80 != 0;
    if !forced_blank {
        for y in 0..160u16 {
            for x in 0..240u16 {
                let color = gba.get_pixel_tile_mode(x, y);
                let r = ((color & 0x1F) as u32 * 255 / 31) << 16;
                let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
                let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                fb_noblend[(y as usize) * 240 + (x as usize)] = r | g | b;
            }
        }
    }

    let mut ppm_orig = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm_orig.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm_orig.push('\n');
    }
    std::fs::write("/tmp/frame200_with_blend.ppm", ppm_orig).unwrap();

    let mut ppm_noblend = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb_noblend[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm_noblend.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm_noblend.push('\n');
    }
    std::fs::write("/tmp/frame200_noblend.ppm", ppm_noblend).unwrap();

    println!("Saved frame200_with_blend.ppm and frame200_noblend.ppm");
    println!("BLDCNT=0x{:04X}, BLDY={}", bldcnt_orig, bldy_orig);

    for y in [0u16, 5, 10, 15] {
        for x in [0u16, 30, 40, 50, 60] {
            let c_blend = fb[y as usize * 240 + x as usize];
            let c_noblend = fb_noblend[y as usize * 240 + x as usize];
            let r1 = (c_blend >> 16) & 0xFF;
            let g1 = (c_blend >> 8) & 0xFF;
            let b1 = c_blend & 0xFF;
            let r2 = (c_noblend >> 16) & 0xFF;
            let g2 = (c_noblend >> 8) & 0xFF;
            let b2 = c_noblend & 0xFF;
            println!(
                "  ({:2},{:2}): blend=({},{},{}) noblend=({},{},{})",
                x, y, r1, g1, b1, r2, g2, b2
            );
        }
    }
}
