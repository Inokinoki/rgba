use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..195 { gba.run_frame_parallel(&mut fb); }

    // Check state
    let dispcnt = u16::from_le_bytes([gba.mem().io()[0], gba.mem().io()[1]]);
    let keyinput = u16::from_le_bytes([gba.mem().io()[0x130], gba.mem().io()[0x131]]);
    let state = gba.mem_mut().read_word(0x02000074);
    println!("Before input: DISPCNT=0x{:04X} state={} KEYINPUT=0x{:04X}", dispcnt, state, keyinput);
    
    // Try pressing Start button
    println!("\n=== Simulating Start button press ===");
    gba.input_mut().press_key(rgba::KeyState::START);
    for frame in 0..60 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = u16::from_le_bytes([gba.mem().io()[0], gba.mem().io()[1]]);
        let state = gba.mem_mut().read_word(0x02000074);
        if frame < 5 || frame % 20 == 0 {
            println!("F{}: DISPCNT=0x{:04X} state={}", frame, dispcnt, state);
        }
    }
    
    gba.input_mut().release_key(rgba::KeyState::START);
    for frame in 0..30 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = u16::from_le_bytes([gba.mem().io()[0], gba.mem().io()[1]]);
        let state = gba.mem_mut().read_word(0x02000074);
        if frame < 5 || frame % 10 == 0 {
            println!("F{}: DISPCNT=0x{:04X} state={}", frame+60, dispcnt, state);
        }
    }
}
