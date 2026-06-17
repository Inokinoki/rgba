use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Press A+START from frame 0
    gba.input_mut().press_key(KeyState::A);
    gba.input_mut().press_key(KeyState::START);

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Search EWRAM for input-related values
    // A+START = bit 0 (A) + bit 3 (START) = 0x0009
    // The input function returns pressed keys as active-high: 0x0009
    // Previous value would be 0x0000 (no keys pressed before)
    // New keys: 0x0009 & ~0x0000 = 0x0009

    println!("=== Searching EWRAM for key values 0x0009, 0x0008, 0x0001 ===");
    for addr in (0x02000000..0x02040000).step_by(2) {
        let hw = gba.mem.read_half(addr);
        if hw == 0x0009 {
            println!("  0x{:08X}: {:04X} (A+START)", addr, hw);
        }
    }

    // Also check IWRAM
    println!("=== Searching IWRAM for key values ===");
    for addr in (0x03000000..0x03008000).step_by(2) {
        let hw = gba.mem.read_half(addr);
        if hw == 0x0009 {
            println!("  0x{:08X}: {:04X} (A+START)", addr, hw);
        }
    }

    // KEYINPUT register value
    println!("\nKEYINPUT = 0x{:04X}", gba.mem.read_half(0x04000130));
}
