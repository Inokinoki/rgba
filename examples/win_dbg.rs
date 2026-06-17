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

    let dispcnt = gba.ppu.get_dispcnt();
    println!("DISPCNT: 0x{:04X}", dispcnt);
    println!(
        "  WIN0: {} WIN1: {} OBJWIN: {}",
        (dispcnt >> 13) & 1,
        (dispcnt >> 14) & 1,
        (dispcnt >> 15) & 1
    );

    let win_vis = gba.ppu.get_window_visibility(3, 3);
    println!("  win_vis at (3,3): 0x{:04X}", win_vis);

    let io = gba.mem.io();
    let winout = u16::from_le_bytes([io[0x4A], io[0x4B]]);
    let winin = u16::from_le_bytes([io[0x48], io[0x49]]);
    println!("  WININ: 0x{:04X}  WINOUT: 0x{:04X}", winin, winout);

    let win0h = u16::from_le_bytes([io[0x40], io[0x41]]);
    let win0v = u16::from_le_bytes([io[0x42], io[0x43]]);
    let win1h = u16::from_le_bytes([io[0x44], io[0x45]]);
    let win1v = u16::from_le_bytes([io[0x46], io[0x47]]);
    println!("  WIN0H: 0x{:04X} WIN0V: 0x{:04X}", win0h, win0v);
    println!("  WIN1H: 0x{:04X} WIN1V: 0x{:04X}", win1h, win1v);

    println!("\n=== Layer check at (3,3) ===");
    for bg in 0..4 {
        let pri = gba.ppu.get_bg_priority(bg);
        let en = gba.ppu.is_bg_enabled(bg);
        let vis = (win_vis & (1 << bg)) != 0;
        let px = gba.get_bg_pixel(&gba.ppu, 0, bg, 3, 3);
        println!("BG{}: pri={} en={} vis={} px={:?}", bg, pri, en, vis, px);
    }

    // Check if window is hiding BG0
    // win_vis bit 0 = BG0 visibility
    println!(
        "\nBG0 vis bit: {} (win_vis & 1 = {})",
        win_vis & 1,
        win_vis & 1
    );
}
