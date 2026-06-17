use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    // Load BIOS directly without fast_bios_boot
    let bios_data = std::fs::read("/tmp/gba_bios.bin").unwrap();
    gba.mem.load_bios(bios_data);
    // DON'T call fast_bios_boot - just check raw BIOS
    
    println!("=== Raw BIOS IRQ dispatcher (0xC4-0x140) ===");
    for off in (0xC4..0x140).step_by(4) {
        let val = gba.mem.bios_read_word(off);
        println!("  {:04X}: {:08X}", off, val);
    }
}
