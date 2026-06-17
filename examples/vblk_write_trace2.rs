use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to frame 200 (past boot)
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Now run one frame and check VBLK at every scanline
    let mut prev = gba.mem.read_word(0x03007FF8);
    let mut changes = Vec::new();
    for line in 0..228 {
        gba.run_scanline();
        let cur = gba.mem.read_word(0x03007FF8);
        if cur != prev {
            changes.push((line, prev, cur));
            prev = cur;
        }
    }
    
    println!("VBLK changes during frame 200:");
    for (line, old, new) in &changes {
        println!("  Line {}: {:08X} -> {:08X} (delta={})", line, old, new, new.wrapping_sub(*old));
    }
    println!("Total changes: {}", changes.len());
}
