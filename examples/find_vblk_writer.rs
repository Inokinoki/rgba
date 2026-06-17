use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Search all IWRAM for the value 0x03007FF8 (VBlank counter address)
    let iwram = gba.mem.iwram();
    for offset in (0..0x8000).step_by(4) {
        let word = u32::from_le_bytes([
            iwram[offset], iwram[offset+1], iwram[offset+2], iwram[offset+3]
        ]);
        if word == 0x03007FF8 {
            println!("  Found 0x03007FF8 at IWRAM+0x{:04X} (0x{:08X})", offset, 0x03000000 + offset);
        }
    }
    
    // Also check ROM for the value
    println!("\nSearching ROM for 0x03007FF8...");
    let rom = gba.mem.rom();
    for offset in (0..rom.len().min(0x800000)).step_by(4) {
        let word = u32::from_le_bytes([
            rom[offset], rom[offset+1], rom[offset+2], rom[offset+3]
        ]);
        if word == 0x03007FF8 {
            println!("  Found at ROM+0x{:08X} (0x{:08X})", offset, 0x08000000 + offset);
        }
    }
}
