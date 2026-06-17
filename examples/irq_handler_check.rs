use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run 200 frames to let the game initialize
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Read the IRQ handler address
    let handler = gba.mem.read_word(0x03007FFC);
    println!("IRQ handler: {:08X}", handler);
    
    // Dump the IRQ handler code (should be THUMB code at the handler address)
    let handler_addr = handler & !1; // Clear THUMB bit
    println!("\nIRQ handler code at {:08X}:", handler_addr);
    for i in 0..32 {
        let byte = gba.mem.read_byte(handler_addr + i);
        print!("{:02X} ", byte);
        if (i + 1) % 16 == 0 { println!(); }
    }
    
    // Also check if there's a VBlank counter increment in the handler
    // The handler should be something like:
    // LDR R0, =0x03007FF8
    // LDR R1, [R0]
    // ADD R1, #1
    // STR R1, [R0]
    // ... clear IF ...
    // BX LR
    println!("\n\nVBLK counter: {:08X}", gba.mem.read_word(0x03007FF8));
}
