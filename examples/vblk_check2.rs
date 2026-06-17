use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to frame 100
    for _ in 0..100 { gba.run_frame_parallel(&mut fb); }
    let v100 = gba.mem.read_word(0x03007FF8);
    println!("Frame 100: VBLK={:08X}", v100);
    
    // Manually set to 0x1000
    {
        let iwram = gba.mem.iwram_mut();
        iwram[0x7FF8] = 0x00;
        iwram[0x7FF9] = 0x10;
        iwram[0x7FFA] = 0x00;
        iwram[0x7FFB] = 0x00;
    }
    
    // Run one frame and check delta
    let before = gba.mem.read_word(0x03007FF8);
    gba.run_frame_parallel(&mut fb);
    let after = gba.mem.read_word(0x03007FF8);
    println!("Frame 101: VBLK {:08X} -> {:08X} (delta={})", before, after, after.wrapping_sub(before));
    
    // Run another
    let before2 = gba.mem.read_word(0x03007FF8);
    gba.run_frame_parallel(&mut fb);
    let after2 = gba.mem.read_word(0x03007FF8);
    println!("Frame 102: VBLK {:08X} -> {:08X} (delta={})", before2, after2, after2.wrapping_sub(before2));
}
