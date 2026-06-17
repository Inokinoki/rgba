use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 250 
    for _ in 0..250 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press START+A
    println!("Pressing START+A at frame 250");
    gba.input.press_key(KeyState::START);
    gba.input.press_key(KeyState::A);

    // Track idle timer
    for frame in 250..300 {
        gba.run_frame_parallel(&mut fb);
        let v50 = gba.mem.read_word(0x02000050);
        let state = gba.mem.read_word(0x02000074);
        if frame % 5 == 0 || frame == 251 {
            println!("Frame {:4}: [50]={:08X} state={:08X}", frame, v50, state);
        }
    }

    // Release
    gba.input.release_key(KeyState::START);
    gba.input.release_key(KeyState::A);
    
    // Continue tracking
    for frame in 300..580 {
        gba.run_frame_parallel(&mut fb);
        let state = gba.mem.read_word(0x02000074);
        if state != 1 || frame % 50 == 0 || frame >= 565 {
            let v50 = gba.mem.read_word(0x02000050);
            println!("Frame {:4}: [50]={:08X} state={:08X}", frame, v50, state);
        }
    }
}
