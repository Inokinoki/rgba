use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Set VBLK to a known value
    let iwram = gba.mem.iwram_mut();
    iwram[0x7FF8] = 0x42;
    iwram[0x7FF9] = 0x00;
    iwram[0x7FFA] = 0x00;
    iwram[0x7FFB] = 0x00;
    
    println!("Set VBLK to 0x42");
    
    // Run one frame
    gba.run_frame_parallel(&mut fb);
    
    let vblk = gba.mem.read_word(0x03007FF8);
    println!("After 1 frame: VBLK={:08X}", vblk);
    
    // Run another frame
    gba.run_frame_parallel(&mut fb);
    let vblk2 = gba.mem.read_word(0x03007FF8);
    println!("After 2 frames: VBLK={:08X}", vblk2);
}
