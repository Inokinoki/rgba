use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run frame by frame and check when screen entries change
    let target_entry = 0xC002; // entry[1] of row 0
    let correct_value = 0xB1DAu16; // what mGBA has
    let wrong_value = 0x018Au16; // what we have

    for frame in 0..300 {
        gba.run_frame_parallel(&mut fb);
        gba.sync_ppu_full();
        let vram = gba.mem.vram();

        // Read entry[1]
        let entry = u16::from_le_bytes([vram[target_entry], vram[target_entry + 1]]);

        if frame % 10 == 0 || entry == correct_value || entry == wrong_value {
            if entry != 0 || frame % 30 == 0 {
                eprintln!("Frame {:3}: entry[1] = 0x{:04X}", frame, entry);
            }
        }

        // Also check when entry[0] first becomes non-zero
        if frame % 30 == 0 {
            let e0 = u16::from_le_bytes([vram[0xC000], vram[0xC001]]);
            let e1 = u16::from_le_bytes([vram[0xC002], vram[0xC003]]);
            let e31 = u16::from_le_bytes([vram[0xC03E], vram[0xC03F]]);
            eprintln!(
                "  entries: [0]=0x{:04X} [1]=0x{:04X} [31]=0x{:04X}",
                e0, e1, e31
            );
        }
    }

    // Now check the final state
    gba.sync_ppu_full();
    let vram = gba.mem.vram();
    eprintln!("\n=== Final state (frame 300) ===");
    for i in 0..32 {
        let addr = 0xC000 + i * 2;
        let entry = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
        eprintln!("  [{:2}] 0x{:04X}", i, entry);
    }
}
