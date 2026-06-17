use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to frame 200
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Read the game's IRQ handler address
    let handler = gba.mem.read_word(0x03007FFC);
    println!("IRQ handler: {:08X}", handler);
    
    // Check the VBlank counter right before and after a frame
    let v_before = gba.mem.read_word(0x03007FF8);
    println!("VBLK before frame: {:08X}", v_before);
    
    // Let me also check: does the counter change DURING a frame or only once?
    // Run the frame step by step
    for line in 0..228 {
        gba.run_scanline();
        let v = gba.mem.read_word(0x03007FF8);
        if v != v_before {
            println!("  VBLK changed at scanline {}: {:08X} (delta={})", line, v, v - v_before);
            if v - v_before >= 2 {
                // Double increment detected!
                break;
            }
        }
    }
}
