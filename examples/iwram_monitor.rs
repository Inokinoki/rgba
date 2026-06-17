use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let iwram_snapshot: Vec<u8> = gba.mem().iwram().to_vec();
    let irq_handler = u32::from_le_bytes([
        iwram_snapshot[0x7FFC],
        iwram_snapshot[0x7FFD],
        iwram_snapshot[0x7FFE],
        iwram_snapshot[0x7FFF],
    ]);
    let vblank_count = u32::from_le_bytes([
        iwram_snapshot[0x7FF8],
        iwram_snapshot[0x7FF9],
        iwram_snapshot[0x7FFA],
        iwram_snapshot[0x7FFB],
    ]);
    println!("=== Frame 200 ===");
    println!("IRQ handler: {:08X}", irq_handler);
    println!("VBlank count: {:08X}", vblank_count);

    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }
    println!("\nFrame 500: DISPCNT={:04X}", gba.ppu().get_dispcnt());

    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let iwram2 = gba.mem().iwram();
    println!("After START: DISPCNT={:04X}", gba.ppu().get_dispcnt());
    println!("\nIWRAM changes (0x4000-0x5000):");
    for i in (0x4000..0x5000).step_by(4) {
        let val = u32::from_le_bytes([iwram2[i], iwram2[i + 1], iwram2[i + 2], iwram2[i + 3]]);
        let old = u32::from_le_bytes([
            iwram_snapshot[i],
            iwram_snapshot[i + 1],
            iwram_snapshot[i + 2],
            iwram_snapshot[i + 3],
        ]);
        if val != old {
            println!("  {:04X}: {:08X} -> {:08X}", i, old, val);
        }
    }

    // Also check IRQ handler at end
    let irq2 = u32::from_le_bytes([
        iwram2[0x7FFC],
        iwram2[0x7FFD],
        iwram2[0x7FFE],
        iwram2[0x7FFF],
    ]);
    println!(
        "\nIRQ handler after START: {:08X} (was {:08X})",
        irq2, irq_handler
    );

    // Check 0x03007C00-0x03007F00 (BIOS work area)
    println!("\nIWRAM BIOS area (0x7C00-0x7F00):");
    for i in (0x7C00..0x7F00).step_by(4) {
        let val = u32::from_le_bytes([iwram2[i], iwram2[i + 1], iwram2[i + 2], iwram2[i + 3]]);
        if val != 0 {
            println!("  {:04X}: {:08X}", i, val);
        }
    }
}
