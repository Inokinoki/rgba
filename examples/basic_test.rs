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
    
    println!("use_real_bios: {}", gba.mem.use_real_bios);
    
    let mut fb = vec![0u32; 240 * 160];
    
    for frame in 0..=10 {
        gba.run_frame_parallel(&mut fb);
        let pc = gba.cpu.get_pc();
        let dispcnt = gba.ppu().get_dispcnt();
        let vblk = gba.mem.read_word(0x03007FF8);
        println!("Frame {:3}: PC={:08X} DISPCNT={:04X} VBLK={:08X}", frame, pc, dispcnt, vblk);
    }
    
    // Run more frames
    for _ in 0..490 { gba.run_frame_parallel(&mut fb); }
    save_png(&fb, "/tmp/basic_test_f500.png");
    println!("\nFrame 500: DISPCNT={:04X} VBLK={:08X}", gba.ppu().get_dispcnt(), gba.mem.read_word(0x03007FF8));
}
