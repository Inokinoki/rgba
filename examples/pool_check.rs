use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Check the literal pool around 0x03000A18-0x03000A60
    for addr in (0x03000A10..0x03000A60).step_by(4) {
        let word = gba.mem.read_word(addr);
        println!("  [{:08X}] = {:08X}", addr, word);
    }
}
