use rgba::Gba;
fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut data = Vec::new();
    for i in (0..64).step_by(4) {
        let w = gba.mem.read_word(0x18 + i as u32);
        data.extend_from_slice(&w.to_le_bytes());
    }
    std::fs::write("/tmp/bios_new.bin", &data).unwrap();
    println!("Dumped {} bytes", data.len());
}
