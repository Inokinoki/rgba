use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem.rom();

    // Disassemble the decompression code at 0x080D0940-0x080D0980
    let base = 0x080D0940;
    let off = (base - 0x08000000) as usize;

    for i in 0..40 {
        let addr = base + i * 2;
        let hw = u16::from_le_bytes([rom[off + i * 2], rom[off + i * 2 + 1]]);
        println!("{:08X}: {:04X}", addr, hw);
    }

    // Also check the broader decompression range
    println!("\n=== Full decompression range hits ===");
    let decomp_base = 0x080D0900;
    let decomp_off = (decomp_base - 0x08000000) as usize;
    for i in 0..0x180 {
        let addr = decomp_base + i * 2;
        let hw = u16::from_le_bytes([rom[decomp_off + i * 2], rom[decomp_off + i * 2 + 1]]);
        println!("{:08X}: {:04X}", addr, hw);
    }
}
