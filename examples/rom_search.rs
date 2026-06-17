use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem.rom();

    // Search for the data that mGBA has at EWRAM 0x02008000
    let targets = [
        (0xFFFF_FFEF_u32, "FFFFFEEF (EWRAM 0x02008000)"),
        (0xF89F_F3FF_u32, "F89FF3FF (EWRAM 0x02008200)"),
        (0x0000_6318_u32, "00006318 (EWRAM 0x0200871C)"),
    ];

    for (target, desc) in &targets {
        println!("Searching for {:08X} ({})...", target, desc);
        let mut found = 0;
        for i in (0..rom.len() - 3).step_by(4) {
            let v = u32::from_le_bytes([rom[i], rom[i + 1], rom[i + 2], rom[i + 3]]);
            if v == *target {
                println!("  Found at ROM {:08X}", 0x08000000 + i);
                found += 1;
                if found >= 5 {
                    break;
                }
            }
        }
        if found == 0 {
            // Try halfword search
            let hw = *target as u16;
            for i in (0..rom.len() - 1).step_by(2) {
                let v = u16::from_le_bytes([rom[i], rom[i + 1]]);
                if v == hw {
                    println!("  (HW match at ROM {:08X}: {:04X})", 0x08000000 + i, hw);
                    break;
                }
            }
        }
    }

    // Also check: what does the CpuSet call on frame 0 actually do?
    // SWI 11 (CpuSet) was called 4 times on frame 0. Let me find those calls.
    println!("\n=== Checking game init data structures ===");

    // The game likely has a table of data to load. Let me check the pointer
    // at 0x020089F8: 0x084D4CC8 - this points to ROM
    let rom_off = 0x084D4CC8 - 0x08000000;
    if rom_off + 32 < rom.len() {
        println!("\nROM at 084D4CC8 (pointer from EWRAM 0x020089F8):");
        for i in 0..8 {
            let off = rom_off + i * 4;
            let v = u32::from_le_bytes([rom[off], rom[off + 1], rom[off + 2], rom[off + 3]]);
            println!("  {:08X}: {:08X}", 0x084D4CC8 + i * 4, v);
        }
    }
}
