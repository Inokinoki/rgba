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

fn test_with_config(name: &str, save_type: rgba::SaveType, zero_sram: bool) {
    println!("\n=== {} ===", name);
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.set_save_type(save_type);
    if zero_sram {
        gba.mem.zero_sram();
    }
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to title screen
    for i in 0..500 { gba.run_frame_parallel(&mut fb); }
    println!("  Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    
    // Check SRAM content
    let first16: Vec<String> = (0..16).map(|i| {
        format!("{:02X}", gba.mem.read_byte(0x0E000000 + i))
    }).collect();
    println!("  SRAM[0..16]: {}", first16.join(" "));
    
    // Press START
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    println!("  After START: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    
    // Press A
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    println!("  After A: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    
    // Press A again
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    println!("  After A2: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    
    save_png(&fb, &format!("/tmp/sram_test_{}.png", name));
    
    // Check if this looks like a dialogue screen or name-entry
    let dispcnt = gba.ppu().get_dispcnt();
    if dispcnt == 0x1F40 {
        println!("  -> Looks like DIALOGUE screen (correct!)");
    } else if dispcnt == 0x1640 {
        println!("  -> Looks like NAME-ENTRY screen (wrong - skipped intro)");
    } else {
        println!("  -> Unknown screen");
    }
}

fn main() {
    test_with_config("flash64_ff", rgba::SaveType::Flash64K, false);
    test_with_config("flash64_00", rgba::SaveType::Flash64K, true);
    test_with_config("flash128_ff", rgba::SaveType::Flash128K, false);
    test_with_config("sram_ff", rgba::SaveType::Sram, false);
    test_with_config("sram_00", rgba::SaveType::Sram, true);
    test_with_config("none", rgba::SaveType::None, false);
}
