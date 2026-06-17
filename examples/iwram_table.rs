use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..200 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    // r7 points to a table at 0x03006994 or 0x03007A00
    // These are in EWRAM (0x02000000-0x0203FFFF) or IWRAM (0x03000000-0x03007FFF)
    // 0x03006994 is in IWRAM, 0x03007A00 is in IWRAM

    let iwram = gba.mem().iwram();

    println!("=== IWRAM at 0x03006994 (table used for frames 188+) ===");
    let base = 0x6994;
    for i in 0..32 {
        let offset = base + i * 8;
        if offset + 8 <= iwram.len() {
            let a = u32::from_le_bytes([
                iwram[offset],
                iwram[offset + 1],
                iwram[offset + 2],
                iwram[offset + 3],
            ]);
            let b = u32::from_le_bytes([
                iwram[offset + 4],
                iwram[offset + 5],
                iwram[offset + 6],
                iwram[offset + 7],
            ]);
            print!("  {:04X}: {:08X} {:08X}", offset, a, b);
            // Try to interpret as decompression entries
            if a >= 0x08000000 {
                print!(" [src={:08X}]", a);
            }
            if b >= 0x06000000 && b < 0x06018000 {
                print!(" [dst=VRAM]");
            }
            if b >= 0x03000000 && b < 0x04000000 {
                print!(" [dst=IWRAM]");
            }
            println!();
        }
    }

    println!("\n=== IWRAM at 0x03007A00 (table used for frames 5-6) ===");
    let base = 0x7A00;
    for i in 0..32 {
        let offset = base + i * 8;
        if offset + 8 <= iwram.len() {
            let a = u32::from_le_bytes([
                iwram[offset],
                iwram[offset + 1],
                iwram[offset + 2],
                iwram[offset + 3],
            ]);
            let b = u32::from_le_bytes([
                iwram[offset + 4],
                iwram[offset + 5],
                iwram[offset + 6],
                iwram[offset + 7],
            ]);
            print!("  {:04X}: {:08X} {:08X}", offset, a, b);
            if a >= 0x08000000 {
                print!(" [src]");
            }
            if b >= 0x06000000 {
                print!(" [dst=VRAM]");
            }
            if b >= 0x03000000 {
                print!(" [dst=IWRAM]");
            }
            println!();
        }
    }

    // Also check what's at r7+4 and r7+6 (the function reads ldrh [r5, #4] and ldrb [r5, #6])
    println!("\n=== Table entry structure ===");
    let base = 0x7A00;
    for i in 0..8 {
        let offset = base + i * 8;
        if offset + 8 <= iwram.len() {
            let w0 = u32::from_le_bytes([
                iwram[offset],
                iwram[offset + 1],
                iwram[offset + 2],
                iwram[offset + 3],
            ]);
            let h4 = u16::from_le_bytes([iwram[offset + 4], iwram[offset + 5]]);
            let b6 = iwram[offset + 6];
            let b7 = iwram[offset + 7];
            println!(
                "  Entry {}: w0={:08X} h4={:04X} b6={:02X} b7={:02X}",
                i, w0, h4, b6, b7
            );
        }
    }
}
