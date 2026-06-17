use rgba::Gba;
use std::fs;
use std::process::Command;

fn save_png(fb: &[u32], path: &str) {
    let ppm = path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let p = fb[y * 240 + x];
            bytes.push(((p >> 16) & 0xFF) as u8);
            bytes.push(((p >> 8) & 0xFF) as u8);
            bytes.push((p & 0xFF) as u8);
        }
    }
    fs::write(&ppm, &bytes).unwrap();
    Command::new("python3")
        .args(["-c", &format!("from PIL import Image; Image.open('{}').save('{}')", ppm, path)])
        .output().unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run 500 frames, press START, run 200 frames, press A, run 200 frames
    // Just like mGBA would
    for _ in 0..500 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/long_500.png");
    println!("Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Press START once and wait 200 frames
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..190 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/long_710.png");
    println!("Frame 710 (after START): DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Press A once and wait 200 frames
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..190 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/long_920.png");
    println!("Frame 920 (after START+A): DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Press A again and wait 200 frames
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..190 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/long_1130.png");
    println!("Frame 1130 (after START+A+A): DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Press A again and wait 200 frames  
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..190 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/long_1340.png");
    println!("Frame 1340 (after START+A+A+A): DISPCNT={:04X}", gba.ppu().get_dispcnt());
}
