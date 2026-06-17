use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 250 (well into title screen)
    for _ in 0..250 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press START+A
    gba.input.press_key(KeyState::START);
    gba.input.press_key(KeyState::A);

    // Check KEYINPUT at the end of each frame  
    for frame in 250..260 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem.io();
        let keyinput = u16::from_le_bytes([io[0x130], io[0x131]]);
        println!("Frame {:4}: KEYINPUT={:04X} (A={} START={} B={} SEL={})", 
            frame, keyinput,
            if keyinput & 1 == 0 { "PRESSED" } else { "up" },
            if keyinput & 8 == 0 { "PRESSED" } else { "up" },
            if keyinput & 2 == 0 { "PRESSED" } else { "up" },
            if keyinput & 4 == 0 { "PRESSED" } else { "up" },
        );
    }
}
