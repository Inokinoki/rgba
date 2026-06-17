use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);

        if frame >= 30 && frame <= 45 {
            let ppu = &gba.ppu;
            let dispcnt = ppu.get_dispcnt();
            let mode = ppu.get_display_mode();
            let hofs = ppu.get_bg_hofs(0);
            let bldy = ppu.get_blend_brightness();
            let bldcnt = ppu.get_blend_control();
            let bg0_enabled = ppu.is_bg_enabled(0);

            let c0 = fb[0];
            let c40_8 = fb[8 * 240 + 40];
            let c80_40 = fb[40 * 240 + 80];

            let fb_r0 = (c0 >> 16) & 0xFF;
            let fb_g0 = (c0 >> 8) & 0xFF;
            let fb_b0 = c0 & 0xFF;

            let fb_r40 = (c40_8 >> 16) & 0xFF;
            let fb_g40 = (c40_8 >> 8) & 0xFF;
            let fb_b40 = c40_8 & 0xFF;

            let unique: std::collections::HashSet<u32> = fb.iter().copied().collect();

            println!("Frame {:3}: dispcnt=0x{:04X} mode={} BG0_en={} hofs={} BLDY={} BLDCNT=0x{:04X} | FB(0,0)=({},{},{}) FB(40,8)=({},{},{}) unique={}", 
                     frame, dispcnt, mode, bg0_enabled, hofs, bldy, bldcnt,
                     fb_r0, fb_g0, fb_b0, fb_r40, fb_g40, fb_b40, unique.len());
        }
    }
}
