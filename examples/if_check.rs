use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    for frame in 0..5 {
        let if_before = gba.mem.read_half(0x04000202);
        let ime_before = gba.mem.read_half(0x04000208);
        let vblk_before = gba.mem.read_word(0x03007FF8);
        gba.run_frame_parallel(&mut fb);
        let if_after = gba.mem.read_half(0x04000202);
        let ime_after = gba.mem.read_half(0x04000208);
        let vblk_after = gba.mem.read_word(0x03007FF8);
        println!("Frame {}: IF {:04X}->{:04X} IME {:04X}->{:04X} VBLK {:08X}->{:08X}",
            frame, if_before, if_after, ime_before, ime_after, vblk_before, vblk_after);
    }
}
