use rgba::{Gba, KeyState};

fn main() {
    let mut fb = vec![0u32; 240 * 160];

    // Test: wait for title screen, then briefly press START (just 1 frame)
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press START for exactly 1 frame
    gba.input_mut().press_key(KeyState::START);
    gba.run_frame_parallel(&mut fb);
    gba.input_mut().release_key(KeyState::START);

    let state_after = gba.mem.read_word(0x02000074);
    println!(
        "After 1-frame START press at frame 200: state={:08X}",
        state_after
    );

    // Check if state changed within the next 400 frames
    for f in 0..400 {
        gba.run_frame_parallel(&mut fb);
        let s = gba.mem.read_word(0x02000074);
        if s != state_after && s != 1 {
            println!("Frame {}: state changed to {:08X}", 201 + f, s);
            break;
        }
    }

    // Test 2: press A for 1 frame at frame 200
    println!("\n=== Press A for 1 frame at frame 200 ===");
    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..200 {
        gba2.run_frame_parallel(&mut fb);
    }
    gba2.input_mut().press_key(KeyState::A);
    gba2.run_frame_parallel(&mut fb);
    gba2.input_mut().release_key(KeyState::A);

    for f in 0..400 {
        gba2.run_frame_parallel(&mut fb);
        let s = gba2.mem.read_word(0x02000074);
        if s != 0 && s != 1 {
            println!("Frame {}: state={:08X}", 201 + f, s);
            break;
        }
    }

    // Test 3: check what the input struct looks like at frame 201 (after START press)
    println!("\n=== Input struct at frame 201 ===");
    let mut gba3 = Gba::new();
    gba3.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba3.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..200 {
        gba3.run_frame_parallel(&mut fb);
    }
    gba3.input_mut().press_key(KeyState::START);
    gba3.run_frame_parallel(&mut fb);
    // Check input struct
    let base = 0x02008CF8;
    for off in (0..0x20).step_by(2) {
        let v = gba3.mem.read_half(base + off);
        if v != 0 {
            println!("  [0x{:02X}] = {:04X}", off, v);
        }
    }
    gba3.input_mut().release_key(KeyState::START);
    gba3.run_frame_parallel(&mut fb);
    println!("After release:");
    for off in (0..0x20).step_by(2) {
        let v = gba3.mem.read_half(base + off);
        if v != 0 {
            println!("  [0x{:02X}] = {:04X}", off, v);
        }
    }
}
