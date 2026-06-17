use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check KEYINPUT register in IO
    let io = gba.mem().io();
    let keyinput = io[0x130] as u16 | ((io[0x131] as u16) << 8);
    println!("KEYINPUT before: {:04X} (bit3=START, 0=pressed)", keyinput);

    gba.input_mut().press_key(rgba::KeyState::START);

    // Run a few steps to sync input
    for _ in 0..1000 {
        gba.step();
    }

    let io = gba.mem().io();
    let keyinput = io[0x130] as u16 | ((io[0x131] as u16) << 8);
    println!("KEYINPUT after press: {:04X}", keyinput);

    // Run rest of frame
    for _ in 0..280796 {
        gba.step();
    }

    println!("Frame done. DISPCNT={:04X}", gba.ppu().get_dispcnt());

    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("After START: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Press A
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("After START+A: DISPCNT={:04X}", gba.ppu().get_dispcnt());
}
