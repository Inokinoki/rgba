use rgba::{Gba, KeyState};

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    println!(
        "Frame 200: state={:08X} timer={:08X}",
        gba.mem.read_word(0x02000074),
        gba.mem.read_word(0x02000050)
    );

    gba.input_mut().press_key(KeyState::A);
    gba.input_mut().press_key(KeyState::START);

    for f in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.input_mut().release_key(KeyState::A);
    gba.input_mut().release_key(KeyState::START);

    println!(
        "Frame 260 (after A+START 60f): state={:08X} timer={:08X}",
        gba.mem.read_word(0x02000074),
        gba.mem.read_word(0x02000050)
    );

    for f in 0..400 {
        gba.run_frame_parallel(&mut fb);
        let s = gba.mem.read_word(0x02000074);
        if s != 1 {
            println!(
                "Frame {}: state={:08X} timer={:08X}",
                260 + f,
                s,
                gba.mem.read_word(0x02000050)
            );
            break;
        }
    }

    // Test 2: Press A+START from frame 0
    println!("\n=== Test 2: A+START from frame 0 ===");
    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    gba2.input_mut().press_key(KeyState::A);
    gba2.input_mut().press_key(KeyState::START);

    for f in 0..700 {
        gba2.run_frame_parallel(&mut fb);
        let s = gba2.mem.read_word(0x02000074);
        if s != 0 && s != 1 {
            println!("Frame {}: state={:08X}", f, s);
            break;
        }
    }
    println!("Frame 700: state={:08X}", gba2.mem.read_word(0x02000074));
}
