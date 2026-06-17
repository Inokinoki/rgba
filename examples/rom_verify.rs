use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // The decompression function reads bitstream data from ROM at r0.
    // The function uses ldmia r0!, {r2} to load 32 bits at a time.
    // Let me verify that read_word returns the correct data for the ROM addresses used.

    let rom = gba.mem().rom();

    // Source addresses from the VRAM decompression calls
    let sources = [
        0x084D4E00u32,
        0x084D4E84,
        0x084D5034,
        0x084D50DC,
        0x084D5180,
        0x084D519C,
        0x084D51C8,
        0x084D5288,
        0x084D12CC,
        0x084D13C4,
        0x084D1608,
        0x084D16FC,
        0x084D17F0,
        0x084D1874,
        0x084D19D8,
        0x084D1A14,
        0x084D1A58,
        0x084D1A90,
        0x084D1B10,
    ];

    println!("ROM data at decompression source addresses:");
    for &src in &sources {
        let offset = (src - 0x08000000) as usize;
        if offset + 16 <= rom.len() {
            let d = &rom[offset..offset + 16];
            println!("  {:08X}: {:02X}{:02X}{:02X}{:02X} {:02X}{:02X}{:02X}{:02X} {:02X}{:02X}{:02X}{:02X} {:02X}{:02X}{:02X}{:02X}",
                     src, d[3], d[2], d[1], d[0], d[7], d[6], d[5], d[4],
                     d[11], d[10], d[9], d[8], d[15], d[14], d[13], d[12]);
        }
    }

    // Now let me also verify that read_word_fast returns the same data
    // The CPU uses read_word_fast for instruction fetch, but for data reads
    // it uses read_word. Let me check both.
    println!("\nVerify read_word vs raw ROM:");
    let src0 = sources[0];
    let src1 = sources[1];
    let src2 = sources[2];
    for src in [src0, src1, src2] {
        let offset = (src - 0x08000000) as usize;
        let raw = u32::from_le_bytes([
            rom[offset],
            rom[offset + 1],
            rom[offset + 2],
            rom[offset + 3],
        ]);
        let read = gba.mem().read_word(src);
        let fast = gba.mem().read_word_fast(src);
        println!(
            "  {:08X}: raw={:08X} read_word={:08X} read_fast={:08X} match={}",
            src,
            raw,
            read,
            fast,
            raw == read && raw == fast
        );
    }
}
