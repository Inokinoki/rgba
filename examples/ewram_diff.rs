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

    // Dump a range of EWRAM addresses to find the game state
    println!("=== EWRAM dump at frame 200 ===");
    for addr in (0x02000000..=0x02000100).step_by(4) {
        let v = gba.mem.read_word(addr);
        if v != 0 {
            println!("0x{:08X}: {:08X}", addr, v);
        }
    }

    // Run to frame 568 and check again
    for _ in 0..368 {
        gba.run_frame_parallel(&mut fb);
    }
    println!("\n=== EWRAM dump at frame 568 ===");
    for addr in (0x02000000..=0x02000100).step_by(4) {
        let v = gba.mem.read_word(addr);
        if v != 0 {
            println!("0x{:08X}: {:08X}", addr, v);
        }
    }
}
