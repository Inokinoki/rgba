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

    // Run to frame 500
    for _ in 0..500 { gba.run_frame_parallel(&mut fb); }
    println!("Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    // Try pressing START many times
    for attempt in 0..10 {
        gba.input_mut().press_key(rgba::KeyState::START);
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        gba.input_mut().release_key(rgba::KeyState::START);
        for _ in 0..60 { gba.run_frame_parallel(&mut fb); }
        let dispcnt = gba.ppu().get_dispcnt();
        println!("  START attempt {}: DISPCNT={:04X}", attempt, dispcnt);
        if dispcnt != 0x1F40 {
            save_png(&fb, &format!("/tmp/start_attempt_{}.png", attempt));
            println!("  State changed! Saved screenshot.");
            break;
        }
    }

    // Reset and try pressing A first (some games have intro screens)
    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb2 = vec![0u32; 240 * 160];
    
    for _ in 0..500 { gba2.run_frame_parallel(&mut fb2); }
    
    // Try pressing A
    gba2.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba2.run_frame_parallel(&mut fb2); }
    gba2.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..60 { gba2.run_frame_parallel(&mut fb2); }
    println!("\nAfter A at frame 500: DISPCNT={:04X}", gba2.ppu().get_dispcnt());

    // Try pressing A many times to skip intro screens
    for i in 0..5 {
        gba2.input_mut().press_key(rgba::KeyState::A);
        for _ in 0..10 { gba2.run_frame_parallel(&mut fb2); }
        gba2.input_mut().release_key(rgba::KeyState::A);
        for _ in 0..60 { gba2.run_frame_parallel(&mut fb2); }
        let dispcnt = gba2.ppu().get_dispcnt();
        println!("  A press {}: DISPCNT={:04X}", i+1, dispcnt);
    }
    
    save_png(&fb2, "/tmp/bios_after_many_a.png");
}
