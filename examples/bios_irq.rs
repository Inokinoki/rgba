use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 200
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check BIOS code around IRQ dispatcher
    println!("=== BIOS code ===");
    // The IRQ handler at 0xC4
    for off in (0xC4..0x120).step_by(4) {
        let val = gba.mem.bios_read_word(off);
        println!("  {:08X}: {:08X}", off, val);
    }
    
    // Check if BIOS has been patched (BX LR returns)
    println!("\n=== BIOS return stubs ===");
    for off in [0x134, 0x138, 0x13C, 0x148] {
        let val = gba.mem.bios_read_word(off);
        println!("  {:08X}: {:08X}", off, val);
    }
}
