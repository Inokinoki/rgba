use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 560 (just before auto-progression)
    for _ in 0..560 {
        gba.run_frame_parallel(&mut fb);
    }

    // Dump memory around 0x02000074
    println!("=== Memory around 0x02000074 at frame 560 ===");
    for off in (0x60..=0xA0).step_by(4) {
        let addr = 0x02000000 + off as u32;
        let val = gba.mem.read_word(addr);
        println!("  {:08X}: {:08X}", addr, val);
    }

    // Also dump some other interesting areas
    println!("\n=== IWRAM near VBlank counter ===");
    for off in (0x7FF0..=0x7FFF).step_by(4) {
        let addr = 0x03000000 + off as u32;
        let val = gba.mem.read_word(addr);
        println!("  {:08X}: {:08X}", addr, val);
    }

    // Timer registers
    println!("\n=== Timer registers ===");
    let io = gba.mem.io();
    for t in 0..4 {
        let base = 0x100 + t * 4;
        let cnt_l = u16::from_le_bytes([io[base], io[base + 1]]);
        let cnt_h = u16::from_le_bytes([io[base + 2], io[base + 3]]);
        println!("  Timer{}: {:04X} {:04X}", t, cnt_l, cnt_h);
    }
}
