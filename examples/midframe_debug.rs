use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..199 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.run_scanline();
    gba.sync_ppu();
    let bldy_scanline0 = gba.ppu.get_blend_brightness();
    let bldcnt_scanline0 = gba.ppu.get_blend_control();

    for scanline in 1..20 {
        gba.run_scanline();
        gba.sync_ppu();
        let bldy = gba.ppu.get_blend_brightness();
        let bldcnt = gba.ppu.get_blend_control();
        let blend_mode = (bldcnt >> 6) & 3;
        println!(
            "Scanline {:3}: BLDY={}, BLDCNT=0x{:04X}, mode={}",
            scanline, bldy, bldcnt, blend_mode
        );
        if bldy != bldy_scanline0 || bldcnt != bldcnt_scanline0 {
            println!("  *** CHANGED from initial values! ***");
        }
    }

    println!("\n=== Checking which BG layer wins at text positions ===");
    for scanline in 20..160 {
        gba.run_scanline();
    }
    gba.sync_ppu_full();

    let ppu = &gba.ppu;
    let mode = ppu.get_display_mode();

    println!("\nText region (x=30-63, y=0-15) - which layer wins:");
    for y in 0..16u16 {
        for x in 30..64u16 {
            let mut first_color = 0u16;
            let mut first_type = "None";
            let mut first_priority = 5u8;

            for bg in 0..4 {
                if ppu.is_bg_enabled(bg) {
                    let priority = ppu.get_bg_priority(bg) as u8;
                    if priority >= first_priority {
                        continue;
                    }
                    if let Some(color) = gba.get_bg_pixel(ppu, mode, bg, x, y) {
                        first_color = color;
                        first_type = match bg {
                            0 => "BG0",
                            1 => "BG1",
                            2 => "BG2",
                            _ => "BG3",
                        };
                        first_priority = priority;
                    }
                }
            }

            if first_type != "None" && x == 40 {
                let r = first_color & 0x1F;
                let g = (first_color >> 5) & 0x1F;
                let b = (first_color >> 10) & 0x1F;
                println!(
                    "  ({}, {}): winner={} (pri={}) color=0x{:04X} (r={} g={} b={})",
                    x, y, first_type, first_priority, first_color, r, g, b
                );
            }
        }
    }

    let bldy_final = gba.ppu.get_blend_brightness();
    let bldcnt_final = gba.ppu.get_blend_control();
    println!("\n=== Final PPU state ===");
    println!(
        "BLDY={}, BLDCNT=0x{:04X}, mode={}",
        bldy_final,
        bldcnt_final,
        (bldcnt_final >> 6) & 3
    );

    let final_color = gba.get_pixel_tile_mode(40, 7);
    let r = final_color & 0x1F;
    let g = (final_color >> 5) & 0x1F;
    let b = (final_color >> 10) & 0x1F;
    println!(
        "Final pixel (40,7): 0x{:04X} (r={} g={} b={})",
        final_color, r, g, b
    );
}
