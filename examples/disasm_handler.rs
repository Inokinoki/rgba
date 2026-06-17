use rgba::Gba;
fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    for _ in 0..240 { let mut fb = vec![0u32; 240*160]; gba.run_frame_parallel(&mut fb); }
    
    let handler_ptr = gba.mem.read_word(0x03007FFC);
    let base = handler_ptr & !1u32;
    
    // Dump 176 bytes of ARM code to binary file
    let mut data = Vec::new();
    for i in (0..176).step_by(4) {
        let w = gba.mem.read_word(base + i as u32);
        data.extend_from_slice(&w.to_le_bytes());
    }
    std::fs::write("/tmp/handler.bin", &data).unwrap();
    
    // Also dump BIOS handler
    let mut bios_data = Vec::new();
    for i in (0..52).step_by(4) {
        let w = gba.mem.read_word(0x18 + i as u32);
        bios_data.extend_from_slice(&w.to_le_bytes());
    }
    std::fs::write("/tmp/bios_handler.bin", &bios_data).unwrap();
    
    println!("Dumped handler at 0x{:08X} ({} bytes)", base, data.len());
    println!("Handler pointer: 0x{:08X} (THUMB bit: {})", handler_ptr, handler_ptr & 1);
}
