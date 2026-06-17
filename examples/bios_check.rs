use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Check BIOS at offset 0x18 (IRQ vector)
    println!("BIOS[0x18..0x30]:");
    for off in (0x18..0x30).step_by(4) {
        let val = gba.mem.bios_read_word(off);
        println!("  {:08X}: {:08X}", 0x00000000 + off, val);
    }

    // Also check BIOS at 0x08 (SWI vector) 
    println!("\nBIOS[0x00..0x20]:");
    for off in (0x00..0x20).step_by(4) {
        let val = gba.mem.bios_read_word(off);
        println!("  {:08X}: {:08X}", off, val);
    }
}
