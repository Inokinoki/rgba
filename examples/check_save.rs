use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }

    // Check ROM header for save type info
    let rom = gba.mem().rom();
    
    // Game title (offset 0xA0)
    let title: String = rom[0xA0..0xAC].iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
    let game_code: String = rom[0xAC..0xB0].iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
    let maker_code: String = rom[0xB0..0xB2].iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
    
    println!("ROM title: '{}'", title);
    println!("Game code: '{}'", game_code);
    println!("Maker code: '{}'", maker_code);
    println!("ROM size: {} bytes", rom.len());
    
    // Check SRAM/Flash state
    println!("\n=== Save memory check ===");
    // Try reading from SRAM area (0x0E000000)
    for addr in [0x0E000000u32, 0x0E000100, 0x0E00FFFF] {
        let val = gba.mem_mut().read_byte(addr);
        println!("SRAM[0x{:08X}] = 0x{:02X}", addr, val);
    }
    
    // Check Flash state
    for addr in [0x0E005555u32, 0x0E002AAA] {
        let val = gba.mem_mut().read_byte(addr);
        println!("Flash[0x{:08X}] = 0x{:02X}", addr, val);
    }
    
    // Check EEPROM state  
    // EEPROM is accessed via ROM area when ROM is small enough
    let eeprom_val = gba.mem_mut().read_half(0x0DFFFF00);
    println!("EEPROM[0x0DFFFF00] = 0x{:04X}", eeprom_val);
    
    // Check what save type our emulator detects
    println!("\nSave type info:");
    
    // Dump the EWRAM "Smsh" structure
    println!("\n=== EWRAM 0x02000C80 structure (save data?) ===");
    for offset in (0..0x200).step_by(4) {
        let addr = 0x02000C80 + offset;
        let val = gba.mem_mut().read_word(addr);
        if val != 0 {
            // Try to interpret as ASCII
            let bytes = val.to_le_bytes();
            let ascii: String = bytes.iter()
                .map(|&b| if b >= 0x20 && b < 0x7F { b as char } else { '.' })
                .collect();
            println!("  +0x{:04X}: 0x{:08X} ({})", offset, val, ascii);
        }
    }
    
    // Check WAITCNT register  
    let waitcnt = u16::from_le_bytes([gba.mem().io()[0x204], gba.mem().io()[0x205]]);
    println!("\nWAITCNT = 0x{:04X}", waitcnt);
    
    // Check if game reads from save during boot
    // Look at EWRAM 0x02000C80 content more carefully
    let magic = gba.mem_mut().read_word(0x02000C80);
    println!("\nSave magic: 0x{:08X} = '{}{}{}{}'", magic,
        (magic & 0xFF) as u8 as char,
        ((magic >> 8) & 0xFF) as u8 as char,
        ((magic >> 16) & 0xFF) as u8 as char,
        ((magic >> 24) & 0xFF) as u8 as char);
}
