use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    for frame in [100, 200, 300, 400, 500, 600, 700] {
        for _ in 0..frame { gba.run_frame_parallel(&mut fb); }
        let vblk = gba.mem.read_word(0x03007FF8);
        println!("Frame {}: VBLK={:08X} (ratio={:.2})", frame, vblk, vblk as f64 / frame as f64);
    }
}
