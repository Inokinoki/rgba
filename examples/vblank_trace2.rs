use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Track VBLK at every frame
    let mut prev_vblk = 0u32;
    for frame in 0..=250 {
        gba.run_frame_parallel(&mut fb);
        let vblk = gba.mem.read_word(0x03007FF8);
        if vblk != prev_vblk || frame <= 5 || frame % 50 == 0 {
            println!("Frame {:3}: VBLK={:08X} (delta={})", frame, vblk, vblk as i32 - prev_vblk as i32);
            prev_vblk = vblk;
        }
    }
}
