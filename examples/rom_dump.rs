use rgba::Gba;

fn main() {
    let rom = std::fs::read("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    // Dump ARM instructions around the tile-loading PCs
    let pcs = [
        0x080D0B74u32,
        0x080D0BEA,
        0x080D0BFA,
        0x08008EEA,
        0x08008F0A,
    ];

    for pc in pcs {
        println!("\n=== ROM at PC={:#010X} ===", pc);
        let offset = (pc - 0x08000000) as usize;
        for i in 0..16 {
            let addr = offset + i * 4;
            if addr + 4 <= rom.len() {
                let opcode =
                    u32::from_le_bytes([rom[addr], rom[addr + 1], rom[addr + 2], rom[addr + 3]]);
                let display_pc = pc + (i as u32) * 4;
                println!("  {:#010X}: {:#010X}", display_pc, opcode);
            }
        }
    }

    // Also dump around 0x080D0800-0x080D0C00 to see the full tile loading code
    println!("\n=== ROM 0x080D0800-0x080D0C00 (tile loading region) ===");
    let base = (0x080D0800 - 0x08000000) as usize;
    for i in (0..0x400).step_by(4) {
        let addr = base + i;
        if addr + 4 <= rom.len() {
            let opcode =
                u32::from_le_bytes([rom[addr], rom[addr + 1], rom[addr + 2], rom[addr + 3]]);
            let display_pc = 0x080D0800u32 + i as u32;
            // Only print non-zero opcodes
            if opcode != 0 {
                println!("  {:#010X}: {:#010X}", display_pc, opcode);
            }
        }
    }

    // Check game header for entry point
    println!("\n=== Game header ===");
    let entry = u32::from_le_bytes([rom[0], rom[1], rom[2], rom[3]]);
    println!("Entry point: {:#010X}", entry);

    // Check game title
    let title: String = rom[0xA0..0xAC].iter().map(|&b| b as char).collect();
    println!("Title: {}", title.trim());
    let gamecode: String = rom[0xAC..0xB0].iter().map(|&b| b as char).collect();
    println!("Game code: {}", gamecode);
}
