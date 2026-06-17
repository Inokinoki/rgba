use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to frame 200
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Check VBLK before and after a single frame
    let v1 = gba.mem.read_word(0x03007FF8);
    println!("Before frame: VBLK={:08X}", v1);
    
    // Run scanline by scanline and check VBLK
    for line in 0..228 {
        gba.run_scanline();
        let v = gba.mem.read_word(0x03007FF8);
        if v != v1 {
            println!("  Line {}: VBLK changed to {:08X} (delta={})", line, v, v - v1);
        }
    }
    
    let v2 = gba.mem.read_word(0x03007FF8);
    println!("After frame: VBLK={:08X} (delta={})", v2, v2.wrapping_sub(v1));
    
    // Now check: does the counter increment at all?
    // Run another frame
    for line in 0..228 {
        gba.run_scanline();
    }
    let v3 = gba.mem.read_word(0x03007FF8);
    println!("After 2nd frame: VBLK={:08X} (delta={})", v3, v3.wrapping_sub(v2));
}
