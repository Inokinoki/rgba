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

    let mut frame = 0u32;
    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;
    }
    println!("Frame {}: DISPCNT={:04X}", frame, gba.ppu().get_dispcnt());

    // Press START
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); frame += 1; }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut fb); frame += 1; }
    println!("Frame {} (after START): DISPCNT={:04X}", frame, gba.ppu().get_dispcnt());
    save_png(&fb, "/tmp/fc2_after_start.png");

    // Press A
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); frame += 1; }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 { gba.run_frame_parallel(&mut fb); frame += 1; }
    println!("Frame {} (after A): DISPCNT={:04X}", frame, gba.ppu().get_dispcnt());
    save_png(&fb, "/tmp/fc2_after_a.png");

    // Press A again
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); frame += 1; }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 { gba.run_frame_parallel(&mut fb); frame += 1; }
    println!("Frame {} (after A2): DISPCNT={:04X}", frame, gba.ppu().get_dispcnt());
    save_png(&fb, "/tmp/fc2_after_a2.png");

    // Press A again
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); frame += 1; }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 { gba.run_frame_parallel(&mut fb); frame += 1; }
    println!("Frame {} (after A3): DISPCNT={:04X}", frame, gba.ppu().get_dispcnt());
    save_png(&fb, "/tmp/fc2_after_a3.png");

    // Press A again
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); frame += 1; }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 { gba.run_frame_parallel(&mut fb); frame += 1; }
    println!("Frame {} (after A4): DISPCNT={:04X}", frame, gba.ppu().get_dispcnt());
    save_png(&fb, "/tmp/fc2_after_a4.png");
}
