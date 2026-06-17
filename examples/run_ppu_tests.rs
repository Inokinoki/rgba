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
    let tests = [
        ("/home/ubuntu/Builds/gba-tests/ppu/hello.gba", "hello"),
        ("/home/ubuntu/Builds/gba-tests/ppu/shades.gba", "shades"),
        ("/home/ubuntu/Builds/gba-tests/ppu/stripes.gba", "stripes"),
    ];
    
    let mut fb = vec![0u32; 240 * 160];
    let mut all_pass = true;
    
    for (path, name) in &tests {
        let mut gba = Gba::new();
        if gba.load_rom_path(path).is_err() {
            println!("SKIP: {} - load error", name);
            continue;
        }
        
        for _ in 0..300 { gba.run_frame_parallel(&mut fb); }
        
        save_png(&fb, &format!("/tmp/ppu_test_{}.png", name));
        
        // Check R12
        let r12 = fb[10 * 240 + 10];
        println!("{}: pixel(10,10)={:08X}", name, r12);
        
        // For shades, check if we see different shades
        let colors: std::collections::HashSet<u32> = fb.iter().copied().collect();
        println!("{}: {} unique colors", name, colors.len());
    }
}
