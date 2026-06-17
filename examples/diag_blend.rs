use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    for _ in 0..3000 {
        gba.run_frame();
    }
    gba.sync_ppu_full();
    gba.sync_ppu();

    let ppu = gba.ppu();
    let bldcnt = ppu.get_blend_control();
    let bldalpha = ppu.get_blend_alpha();
    let bldy = ppu.get_blend_brightness();
    let blend_mode = ppu.get_blend_mode();

    eprintln!(
        "BLDCNT=0x{:04X} BLDALPHA=0x{:04X} BLDY=0x{:04X} blend_mode={}",
        bldcnt, bldalpha, bldy, blend_mode
    );
    eprintln!(
        "  First target BGs: {:04b} OBJ: {} BD: {}",
        bldcnt & 0xF,
        (bldcnt >> 4) & 1,
        (bldcnt >> 5) & 1
    );
    eprintln!(
        "  Second target BGs: {:04b} OBJ: {} BD: {}",
        (bldcnt >> 8) & 0xF,
        (bldcnt >> 12) & 1,
        (bldcnt >> 13) & 1
    );

    eprintln!(
        "BG mosaic: 0x{:04X} (H={}, V={})",
        ppu.bg_mosaic,
        ppu.bg_mosaic & 0xF,
        (ppu.bg_mosaic >> 4) & 0xF
    );
    eprintln!("OBJ mosaic: 0x{:04X}", ppu.obj_mosaic);

    // Window settings
    eprintln!("Windows: (private fields, skipped)");

    // Check window visibility for a few pixels
    for y in [0u16, 40, 80, 120, 159] {
        for x in [0u16, 120, 239] {
            let vis = ppu.get_window_visibility(x, y);
            eprintln!(
                "  Visibility at ({}, {}): 0x{:04X} (BG0-3: {:04b} OBJ: {})",
                x,
                y,
                vis,
                vis & 0xF,
                (vis >> 4) & 1
            );
        }
    }

    // Test render: check what get_pixel_tile_mode returns for a few points
    eprintln!("\nPixel samples:");
    for y in [0u16, 40, 80, 120, 159] {
        for x in [0u16, 120, 239] {
            let c = gba.get_pixel_tile_mode(x, y);
            let r = c & 0x1F;
            let g = (c >> 5) & 0x1F;
            let b = (c >> 10) & 0x1F;
            eprintln!("  ({}, {}) = 0x{:04X} (r={} g={} b={})", x, y, c, r, g, b);
        }
    }
}
