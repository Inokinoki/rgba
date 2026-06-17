use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();

    let ppu = &gba.ppu;
    let dispcnt = ppu.get_dispcnt();
    eprintln!("DISPCNT: 0x{:04X}", dispcnt);
    eprintln!("  BG mode: {}", dispcnt & 7);
    eprintln!(
        "  BG0-3: {}{}{}{}",
        (dispcnt >> 8) & 1,
        (dispcnt >> 9) & 1,
        (dispcnt >> 10) & 1,
        (dispcnt >> 11) & 1
    );
    eprintln!("  OBJ: {}", (dispcnt >> 12) & 1);
    eprintln!("  Win0: {}", (dispcnt >> 13) & 1);
    eprintln!("  Win1: {}", (dispcnt >> 14) & 1);
    eprintln!("  WinOBJ: {}", (dispcnt >> 15) & 1);

    let io = gba.mem.io();
    let win0h = u16::from_le_bytes([io[0x40], io[0x41]]);
    let win0v = u16::from_le_bytes([io[0x42], io[0x43]]);
    let win1h = u16::from_le_bytes([io[0x44], io[0x45]]);
    let win1v = u16::from_le_bytes([io[0x46], io[0x47]]);
    let winin = u16::from_le_bytes([io[0x48], io[0x49]]);
    let winout = u16::from_le_bytes([io[0x4A], io[0x4B]]);
    let bldcnt = u16::from_le_bytes([io[0x50], io[0x51]]);
    let bldalpha = u16::from_le_bytes([io[0x52], io[0x53]]);
    let bldy = u16::from_le_bytes([io[0x54], io[0x55]]);

    eprintln!(
        "\nWIN0H: 0x{:04X} (L={} R={})",
        win0h,
        win0h & 0xFF,
        (win0h >> 8) & 0xFF
    );
    eprintln!(
        "WIN0V: 0x{:04X} (T={} B={})",
        win0v,
        win0v & 0xFF,
        (win0v >> 8) & 0xFF
    );
    eprintln!("WIN1H: 0x{:04X}", win1h);
    eprintln!("WIN1V: 0x{:04X}", win1v);
    eprintln!(
        "WININ: 0x{:04X} (win0={:05b} win1={:05b})",
        winin,
        winin & 0x1F,
        (winin >> 8) & 0x1F
    );
    eprintln!(
        "WINOUT: 0x{:04X} (outside={:05b} objwin={:05b})",
        winout,
        winout & 0x1F,
        (winout >> 8) & 0x1F
    );
    eprintln!("BLDCNT: 0x{:04X}", bldcnt);
    eprintln!(
        "  Target1 BG: {}{}{}{}",
        (bldcnt >> 0) & 1,
        (bldcnt >> 1) & 1,
        (bldcnt >> 2) & 1,
        (bldcnt >> 3) & 1
    );
    eprintln!("  Target1 OBJ: {}", (bldcnt >> 4) & 1);
    eprintln!("  Target1 BD: {}", (bldcnt >> 5) & 1);
    eprintln!("  Effect: {}", (bldcnt >> 6) & 3);
    eprintln!(
        "  Target2 BG: {}{}{}{}",
        (bldcnt >> 8) & 1,
        (bldcnt >> 9) & 1,
        (bldcnt >> 10) & 1,
        (bldcnt >> 11) & 1
    );
    eprintln!("  Target2 OBJ: {}", (bldcnt >> 12) & 1);
    eprintln!("  Target2 BD: {}", (bldcnt >> 13) & 1);
    eprintln!(
        "BLDALPHA: 0x{:04X} (eva={} evb={})",
        bldalpha,
        bldalpha & 0x1F,
        (bldalpha >> 8) & 0x1F
    );
    eprintln!("BLDY: 0x{:04X}", bldy);

    eprintln!("\nWindow visibility:");
    eprintln!("  (0,0): {:05b}", ppu.get_window_visibility(0, 0));
    eprintln!("  (120,80): {:05b}", ppu.get_window_visibility(120, 80));
    eprintln!("  (239,159): {:05b}", ppu.get_window_visibility(239, 159));

    // Trace one pixel through get_bg_pixel
    let x = 120u16;
    let y = 80u16;
    eprintln!("\n=== Pixel ({},{}) ===", x, y);
    for bg in 0..4 {
        let color = gba.get_bg_pixel(ppu, 0, bg, x, y);
        eprintln!(
            "  BG{} pixel: {:?}",
            bg,
            color.map(|c| format!("0x{:04X}", c))
        );
    }
    let final_color = gba.get_pixel_tile_mode(x, y);
    eprintln!("  Final: 0x{:04X}", final_color);
    let backdrop = gba.mem.read_palette_color(0, 0);
    eprintln!("  Backdrop: 0x{:04X}", backdrop);
}
