use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..600 { gba.run_frame_parallel(&mut fb); }
    
    println!("=== Our EWRAM at frame 600 ===");
    for offset in (0..0x100).step_by(4) {
        let addr = 0x02000000 + offset as u32;
        let val = gba.mem.read_word(addr);
        println!("  [{:08X}] = 0x{:08X}", addr, val);
    }
}
