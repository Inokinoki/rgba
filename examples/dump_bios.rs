use rgba::Gba;
fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    println!("BIOS handler (raw bytes 0x18-0x53):");
    for i in (0..60).step_by(4) {
        let w = gba.mem.read_word(0x18 + i as u32);
        println!("  0x{:04X}: 0x{:08X}", 0x18 + i, w);
    }
}
