use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Search for 0x03007FF8 in a wider range around the handler
    let handler = gba.mem.read_word(0x03007FFC) & !1;
    println!("Handler: {:08X}", handler);
    
    for addr in (handler..handler + 512).step_by(4) {
        let word = gba.mem.read_word(addr);
        if word == 0x03007FF8 || word == 0x03007FFC {
            println!("  Found {:08X} at {:08X}", word, addr);
        }
    }
    
    // The handler might use R3 (loaded from [R3,#0]) which could be a pointer to a struct
    // containing the VBlank counter. Check what R3 points to.
    // From the disassembly: LDR R0, [R3, #0] at offset 0x10
    // And later: LDR R3, [PC, #0x68] at offset 0x3C
    // Which loads from PC+0x68+8 = 0x44+0x68 = 0xAC
    
    // Check data pool at handler+0xA0 to handler+0xC0
    println!("\nData pool:");
    for i in (0xA0..0xC0).step_by(4) {
        let word = gba.mem.read_word(handler + i as u32);
        println!("  [{:08X}] = {:08X}", handler + i as u32, word);
    }
}
