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
    
    for frame in 0..=800 {
        gba.run_frame_parallel(&mut fb);
        let val74 = gba.mem.read_word(0x02000074);
        let val7c = gba.mem.read_word(0x0200007C);
        if frame % 100 == 0 || val74 != 0x00000001 {
            let vblk = gba.mem.read_word(0x03007FF8);
            println!("Frame {:3}: [0074]={:08X} [007C]={:08X} VBLK={:08X} DISPCNT={:04X}", 
                frame, val74, val7c, vblk, gba.ppu().get_dispcnt());
        }
        if val74 != 0x00000001 && frame > 100 {
            save_png(&fb, &format!("/tmp/state_chg_f{}.png", frame));
        }
    }
}
