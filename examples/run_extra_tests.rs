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

fn run_save_test(path: &str, name: &str, save_type: rgba::SaveType) {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    if gba.load_rom_path(path).is_err() {
        println!("SKIP: {} (load error)", name);
        return;
    }
    gba.set_save_type(save_type);
    let mut fb = vec![0u32; 240 * 160];
    
    for _ in 0..300 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, &format!("/tmp/test_save_{}.png", name));
    
    // Check result - same logic as run_tests
    let mut red_count = 0;
    let mut green_count = 0;
    for y in 140..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            let r = (pixel >> 16) & 0xFF;
            let g = (pixel >> 8) & 0xFF;
            let b = pixel & 0xFF;
            if r > 200 && g < 50 && b < 50 { red_count += 1; }
            if g > 200 && r < 50 && b < 50 { green_count += 1; }
        }
    }
    if red_count > green_count && red_count > 100 {
        println!("FAIL: {} (red pixels dominant)", name);
    } else {
        println!("PASS: {} (green={}) (red={})", name, green_count, red_count);
    }
}

fn run_ppu_test(path: &str, name: &str) {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    if gba.load_rom_path(path).is_err() {
        println!("SKIP: {} (load error)", name);
        return;
    }
    let mut fb = vec![0u32; 240 * 160];
    
    for _ in 0..300 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, &format!("/tmp/test_ppu_{}.png", name));
    
    // Check if rendering produces expected output
    let mut non_black = 0;
    let mut non_white = 0;
    for y in 0..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            if pixel != 0 { non_black += 1; }
            if pixel != 0xFFFFFF { non_white += 1; }
        }
    }
    println!("PPU {}: non_black={}, non_white={}", name, non_black, non_white);
}

fn main() {
    println!("=== Save Tests ===");
    run_save_test("/home/ubuntu/Builds/gba-tests/save/none.gba", "none", rgba::SaveType::None);
    run_save_test("/home/ubuntu/Builds/gba-tests/save/sram.gba", "sram", rgba::SaveType::Sram);
    run_save_test("/home/ubuntu/Builds/gba-tests/save/flash64.gba", "flash64", rgba::SaveType::Flash64K);
    run_save_test("/home/ubuntu/Builds/gba-tests/save/flash128.gba", "flash128", rgba::SaveType::Flash128K);
    
    println!("\n=== PPU Tests ===");
    run_ppu_test("/home/ubuntu/Builds/gba-tests/ppu/hello.gba", "hello");
    run_ppu_test("/home/ubuntu/Builds/gba-tests/ppu/shades.gba", "shades");
    run_ppu_test("/home/ubuntu/Builds/gba-tests/ppu/stripes.gba", "stripes");
}
