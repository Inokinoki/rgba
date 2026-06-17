use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    let handler = gba.mem.read_word(0x03007FFC);
    let handler_addr = handler & !1;
    println!("IRQ handler: {:08X} (addr={:08X})", handler, handler_addr);
    
    // Dump 128 bytes of ARM code from the handler
    println!("\nHandler ARM disassembly bytes:");
    for i in 0..128 {
        let b = gba.mem.read_byte(handler_addr + i);
        print!("{:02X} ", b);
        if (i + 1) % 4 == 0 { print!(" "); }
        if (i + 1) % 16 == 0 { println!(); }
    }
    
    // Also check what value 0x03007FF8 has, and check if the handler 
    // references this address
    println!("\n\nSearching for 0x03007FF8 in handler region...");
    for i in (0..128).step_by(4) {
        let word = gba.mem.read_word(handler_addr + i as u32);
        if word == 0x03007FF8 || word == 0x03007FFC || word == 0x04000200 || word == 0x04000202 {
            println!("  Found {:08X} at offset +{}", word, i);
        }
    }
}
