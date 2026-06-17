use rgba::{Gba, KeyState};

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.input_mut().press_key(KeyState::A);
    gba.input_mut().press_key(KeyState::B);
    gba.input_mut().press_key(KeyState::SELECT);
    gba.input_mut().press_key(KeyState::START);

    println!("=== All 4 keys pressed from frame 0 ===");
    for f in 0..800 {
        let s = gba.mem.read_word(0x02000074);
        let t = gba.mem.read_word(0x02000050);
        if f % 50 == 0 || s != 1 {
            println!("Frame {}: state={:08X} timer={:08X}", f, s, t);
        }
        if s != 0 && s != 1 {
            break;
        }
        gba.run_frame_parallel(&mut fb);
    }
}
