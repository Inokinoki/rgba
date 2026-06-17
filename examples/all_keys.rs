use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..250 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press ALL keys in KEYCNT mask: A+B+SELECT+START
    println!("Pressing A+B+SELECT+START (KEYCNT mask)");
    gba.input.press_key(KeyState::A);
    gba.input.press_key(KeyState::B);
    gba.input.press_key(KeyState::SELECT);
    gba.input.press_key(KeyState::START);

    for frame in 250..300 {
        gba.run_frame_parallel(&mut fb);
        let state = gba.mem.read_word(0x02000074);
        let v50 = gba.mem.read_word(0x02000050);
        if state != 1 || frame % 10 == 0 {
            println!("Frame {:4}: state={:08X} [50]={:08X}", frame, state, v50);
        }
    }
}
