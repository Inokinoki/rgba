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
    
    // Run to title screen
    for _ in 0..500 { gba.run_frame_parallel(&mut fb); }
    println!("Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    
    // Press START
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    println!("Frame 710 (after START): DISPCNT={:04X} PC={:08X}", gba.ppu().get_dispcnt(), gba.cpu.get_pc());
    save_png(&fb, "/tmp/dflow_after_start.png");
    
    // Press A
    gba.input_mut().press_key(rgba::KeyState::A);
    for f in 0..300 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = gba.ppu().get_dispcnt();
        if f < 10 || f % 50 == 0 || dispcnt != 0x1F40 {
            println!("  Frame {}: DISPCNT={:04X} PC={:08X}", 710+f, dispcnt, gba.cpu.get_pc());
        }
        if dispcnt == 0x1640 {
            save_png(&fb, &format!("/tmp/dflow_name_entry_f{}.png", 710+f));
            break;
        }
    }
    
    // Take a screenshot of final state
    save_png(&fb, "/tmp/dflow_final.png");
    println!("\nFinal: DISPCNT={:04X} PC={:08X}", gba.ppu().get_dispcnt(), gba.cpu.get_pc());
    
    // Let me also check: what does the screen look like at DISPCNT=1F40 after A press?
    // Is it still the title screen, or something else?
    println!("\nChecking screen content...");
    let mut non_white = 0;
    let mut non_black = 0;
    for y in 0..160 {
        for x in 0..240 {
            let p = fb[y * 240 + x];
            if p != 0xFFFFFF { non_white += 1; }
            if p != 0 { non_black += 1; }
        }
    }
    println!("  Non-white pixels: {}, Non-black pixels: {}", non_white, non_black);
}
