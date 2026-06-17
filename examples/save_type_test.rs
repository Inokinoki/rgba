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
    // Test with Flash 64K save type
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.set_save_type(rgba::SaveType::Flash64K);
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..500 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/flash64_f500.png");
    println!("Flash64K Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/flash64_after_start.png");
    println!("Flash64K After START: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/flash64_dialogue.png");
    println!("Flash64K After START+A: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    {
        let ppu = gba.ppu();
        println!("  BG0CNT={:04X} BG1CNT={:04X} BG2CNT={:04X} BG3CNT={:04X}",
            ppu.get_bgcnt(0), ppu.get_bgcnt(1), ppu.get_bgcnt(2), ppu.get_bgcnt(3));
    }

    // Test with SRAM save type
    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba2.set_save_type(rgba::SaveType::Sram);
    let mut fb2 = vec![0u32; 240 * 160];

    for _ in 0..500 { gba2.run_frame_parallel(&mut fb2); }
    gba2.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba2.run_frame_parallel(&mut fb2); }
    gba2.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 { gba2.run_frame_parallel(&mut fb2); }
    gba2.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba2.run_frame_parallel(&mut fb2); }
    gba2.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..200 { gba2.run_frame_parallel(&mut fb2); }
    println!("\nSRAM After START+A: DISPCNT={:04X}", gba2.ppu().get_dispcnt());
}
