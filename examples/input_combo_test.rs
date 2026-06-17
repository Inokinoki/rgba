use rgba::{Gba, KeyState};

fn main() {
    // Test: press all KEYCNT mask keys (A+B+SELECT+START) from frame 0
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.input_mut().press_key(KeyState::A);
    gba.input_mut().press_key(KeyState::B);
    gba.input_mut().press_key(KeyState::SELECT);
    gba.input_mut().press_key(KeyState::START);

    for f in 0..700 {
        gba.run_frame_parallel(&mut fb);
        let s = gba.mem.read_word(0x02000074);
        if s != 0 && s != 1 {
            println!("Test1 (all keys from 0): Frame {} state={:08X}", f, s);
            break;
        }
    }

    // Test: don't press anything, check when auto-progress happens
    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for f in 0..700 {
        gba2.run_frame_parallel(&mut fb);
        let s = gba2.mem.read_word(0x02000074);
        if s != 0 && s != 1 {
            println!("Test2 (no keys): Frame {} state={:08X}", f, s);
            break;
        }
    }

    // Test: press A+START only
    let mut gba3 = Gba::new();
    gba3.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba3.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    gba3.input_mut().press_key(KeyState::A);
    gba3.input_mut().press_key(KeyState::START);

    for f in 0..700 {
        gba3.run_frame_parallel(&mut fb);
        let s = gba3.mem.read_word(0x02000074);
        if s != 0 && s != 1 {
            println!("Test3 (A+START from 0): Frame {} state={:08X}", f, s);
            break;
        }
    }

    // Test: press nothing for 200 frames, then A+START
    let mut gba4 = Gba::new();
    gba4.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba4.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..200 {
        gba4.run_frame_parallel(&mut fb);
    }
    gba4.input_mut().press_key(KeyState::A);
    gba4.input_mut().press_key(KeyState::START);

    for f in 0..700 {
        gba4.run_frame_parallel(&mut fb);
        let s = gba4.mem.read_word(0x02000074);
        if s != 0 && s != 1 {
            println!(
                "Test4 (A+START from 200): Frame {} state={:08X}",
                200 + f,
                s
            );
            break;
        }
    }
}
