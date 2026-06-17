use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut last_pc_in_range = 0u32;
    let mut call_idx = 0u32;

    for frame in 0..200 {
        for _scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            if pc >= 0x080D0B54 && pc < 0x080D0C10 && last_pc_in_range < 0x080D0B54 {
                let r = gba.cpu().registers();
                if r[1] >= 0x06000000 && r[1] < 0x06018000 {
                    let rom_off = (r[0] - 0x08000000) as usize;
                    let rom = gba.mem().rom();
                    let header = if rom_off + 4 <= rom.len() {
                        u32::from_le_bytes([
                            rom[rom_off],
                            rom[rom_off + 1],
                            rom[rom_off + 2],
                            rom[rom_off + 3],
                        ])
                    } else {
                        0
                    };
                    let header2 = if rom_off + 8 <= rom.len() {
                        u32::from_le_bytes([
                            rom[rom_off + 4],
                            rom[rom_off + 5],
                            rom[rom_off + 6],
                            rom[rom_off + 7],
                        ])
                    } else {
                        0
                    };
                    println!(
                        "VRAM call {}: frame={} src={:08X} dst={:08X} r3={} header={:08X} {:08X}",
                        call_idx, frame, r[0], r[1], r[3], header, header2
                    );
                    call_idx += 1;
                }
            }
            if pc >= 0x080D0B54 && pc < 0x080D0C10 {
                last_pc_in_range = pc;
            } else {
                last_pc_in_range = 0;
            }
            gba.run_scanline();
        }
    }

    // Now check: is there data in ROM that should have been loaded but wasn't?
    // The BG0 screen block references tiles 187-689, starting around tile 187
    // Tile 187 = offset 0x1770 in char block 0
    // Let's check if there's compressed data in ROM for these tiles

    println!("\n=== ROM data analysis ===");
    let rom = gba.mem().rom();

    // Search for compressed tile data headers near the end of used ROM addresses
    // The decompression sources are at 0x084D4E00-0x084D5288 for VRAM writes
    // Let's check the wider area
    let search_start = 0x004D0000;
    let search_end = 0x004E0000;
    for offset in (search_start..search_end).step_by(4) {
        if offset + 4 <= rom.len() {
            let val = u32::from_le_bytes([
                rom[offset],
                rom[offset + 1],
                rom[offset + 2],
                rom[offset + 3],
            ]);
            // Look for patterns that might be tile data headers
            // Common patterns: size followed by type, or just raw data markers
        }
    }

    // Check what ROM addresses are used as decompression sources
    println!("\nROM addresses used as decompression sources (from VRAM calls):");
    // Already printed above
}
