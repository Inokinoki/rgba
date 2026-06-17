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

fn dump_ewram(gba: &mut Gba, path: &str) {
    let mut data = vec![0u8; 0x40000];
    for i in (0..0x40000).step_by(4) {
        let val = gba.mem.read_word(0x02000000 + i as u32);
        data[i..i+4].copy_from_slice(&val.to_le_bytes());
    }
    fs::write(path, &data).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to title screen
    for _ in 0..500 { gba.run_frame_parallel(&mut fb); }
    println!("Frame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    dump_ewram(&mut gba, "/tmp/ewram_f500.bin");
    
    // Press START
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    println!("Frame 710 (after START): DISPCNT={:04X}", gba.ppu().get_dispcnt());
    dump_ewram(&mut gba, "/tmp/ewram_f710.bin");
    
    // Dump at the transition point - frame by frame
    gba.input_mut().press_key(rgba::KeyState::A);
    for f in 0..10 { gba.run_frame_parallel(&mut fb); }
    gba.input_mut().release_key(rgba::KeyState::A);
    
    for f in 0..200 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = gba.ppu().get_dispcnt();
        if dispcnt == 0x0080 && f < 5 {
            println!("  Frame {}: DISPCNT={:04X} (transition to forced blank)", 720+f, dispcnt);
            dump_ewram(&mut gba, &format!("/tmp/ewram_f{}.bin", 720+f));
        }
        if dispcnt == 0x1640 {
            println!("  Frame {}: DISPCNT={:04X} (name entry!)", 720+f, dispcnt);
            dump_ewram(&mut gba, &format!("/tmp/ewram_f{}.bin", 720+f));
            break;
        }
    }
    
    // Compare EWRAM at title screen vs at name entry
    let ewram_title = fs::read("/tmp/ewram_f500.bin").unwrap();
    let ewram_start = fs::read("/tmp/ewram_f710.bin").unwrap();
    
    // Find differences
    println!("\n=== EWRAM differences (title vs after START) ===");
    let mut diff_count = 0;
    for i in (0..0x40000).step_by(4) {
        let v1 = u32::from_le_bytes([ewram_title[i], ewram_title[i+1], ewram_title[i+2], ewram_title[i+3]]);
        let v2 = u32::from_le_bytes([ewram_start[i], ewram_start[i+1], ewram_start[i+2], ewram_start[i+3]]);
        if v1 != v2 && diff_count < 30 {
            println!("  [{:08X}]: {:08X} -> {:08X}", 0x02000000 + i, v1, v2);
            diff_count += 1;
        }
    }
    println!("  Total different words: (checking...)");
    let total_diff: usize = (0..0x40000).step_by(4)
        .filter(|i| {
            let i = *i;
            ewram_title[i..i+4] != ewram_start[i..i+4]
        }).count();
    println!("  Total: {} words different", total_diff);
}
