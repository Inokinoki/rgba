use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..10 { gba.run_frame_parallel(&mut fb); }

    // Dump EWRAM in hex for comparison with mGBA
    println!("=== EWRAM dump at frame 10 ===");
    for offset in (0..0x100).step_by(16) {
        let addr = 0x02000000 + offset;
        let mut words = Vec::new();
        for i in 0..4 {
            words.push(gba.mem_mut().read_word(addr + i * 4));
        }
        print!("0x{:08X}: ", addr);
        for w in &words {
            print!("{:08X} ", w);
        }
        println!();
    }
    
    // Also dump IWRAM key area
    println!("\n=== IWRAM dump at 0x03007F00 ===");
    for offset in (0xF00..0x800).step_by(16) {
        if offset > 0xF80 { break; }
        let addr = 0x03000000 + offset;
        let mut words = Vec::new();
        for i in 0..4 {
            words.push(gba.mem_mut().read_word(addr + i * 4));
        }
        print!("0x{:08X}: ", addr);
        for w in &words {
            print!("{:08X} ", w);
        }
        println!();
    }
}
