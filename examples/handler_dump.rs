use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..2 {
        gba.run_frame_parallel(&mut fb);
    }

    let iwram = gba.mem().iwram();
    let base = 0x03000958 - 0x03000000;
    println!("Game IRQ handler at 0x03000958:");
    for i in (0..64).step_by(4) {
        let word = u32::from_le_bytes([
            iwram[base + i],
            iwram[base + i + 1],
            iwram[base + i + 2],
            iwram[base + i + 3],
        ]);
        println!("  0x{:08X}: 0x{:08X}", 0x03000958 + i, word);
    }

    println!("\nKey memory:");
    println!("[0x03000008] = 0x{:02X}", iwram[0x08]);
    println!(
        "[0x0300000C] = 0x{:08X}",
        u32::from_le_bytes([iwram[0x0C], iwram[0x0D], iwram[0x0E], iwram[0x0F]])
    );
    println!(
        "[0x03000010] = 0x{:08X}",
        u32::from_le_bytes([iwram[0x10], iwram[0x11], iwram[0x12], iwram[0x13]])
    );

    // Also check what value R1 holds when SWI 0x04 is called
    // R1 is the flag pointer for IntrWait
    println!("\nEWRAM key values:");
    let wram = gba.mem().wram();
    for off in [0xC80, 0xC84, 0xC88, 0xC8C].iter() {
        let word = u32::from_le_bytes([wram[*off], wram[*off + 1], wram[*off + 2], wram[*off + 3]]);
        println!("[0x0200{:04X}] = 0x{:08X}", off, word);
    }
}
