use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..8 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let iwram = gba.mem().iwram();

    // Dump literal pool values used by the handler
    // 0x03000994: LDR R1, [PC, #116] → loads from 0x03000A10
    // 0x030009A4: LDR R3, [PC, #104] → loads from 0x03000A14
    // 0x030009C0: LDR R2, [PC, #80]  → loads from 0x03000A10... wait

    // Let me just dump the area after the handler code
    println!("=== Handler data area 0x030009F8-0x03000A60 ===");
    for i in 0..26 {
        let off = 0x9F8 + i * 4;
        if off + 4 <= iwram.len() {
            let word = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
            println!("  {:#010X}: {:#010X}", 0x03000000 + off, word);
        }
    }

    // Check callback table
    // The handler loads callback table ptr from 0x03000A14
    let cb_table_ptr_off = 0xA14;
    if cb_table_ptr_off + 4 <= iwram.len() {
        let cb_table_ptr = u32::from_le_bytes([
            iwram[cb_table_ptr_off], iwram[cb_table_ptr_off+1],
            iwram[cb_table_ptr_off+2], iwram[cb_table_ptr_off+3]
        ]);
        println!("\nCallback table pointer (at 0x03000A14): {:#010X}", cb_table_ptr);

        // Dump callback table entries
        if cb_table_ptr >= 0x03000000 && cb_table_ptr < 0x03008000 {
            let cb_off = (cb_table_ptr - 0x03000000) as usize;
            println!("Callback table entries:");
            for i in 0..14 {
                let off = cb_off + i * 4;
                if off + 4 <= iwram.len() {
                    let cb = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
                    let name = match i {
                        0 => "VBlank", 1 => "HBlank", 2 => "VCounter",
                        3 => "Timer0", 4 => "Timer1", 5 => "Timer2", 6 => "Timer3",
                        7 => "Serial", 8 => "DMA0", 9 => "DMA1", 10 => "DMA2",
                        11 => "DMA3", 12 => "Keypad", 13 => "GamePak",
                        _ => "?"
                    };
                    println!("  [{}] ({:8}): {:#010X}", i, name, cb);
                }
            }
        } else if cb_table_ptr >= 0x02000000 && cb_table_ptr < 0x03000000 {
            let cb_off = (cb_table_ptr - 0x02000000) as usize;
            let wram = gba.mem().wram();
            println!("Callback table entries (in EWRAM):");
            for i in 0..14 {
                let off = cb_off + i * 4;
                if off + 4 <= wram.len() {
                    let cb = u32::from_le_bytes([wram[off], wram[off+1], wram[off+2], wram[off+3]]);
                    let name = match i {
                        0 => "VBlank", 1 => "HBlank", 2 => "VCounter",
                        3 => "Timer0", 4 => "Timer1", 5 => "Timer2", 6 => "Timer3",
                        7 => "Serial", 8 => "DMA0", 9 => "DMA1", 10 => "DMA2",
                        11 => "DMA3", 12 => "Keypad", 13 => "GamePak",
                        _ => "?"
                    };
                    println!("  [{}] ({:8}): {:#010X}", i, name, cb);
                }
            }
        }
    }

    // Also check: what's the constant at 0x03000A10?
    let const_off = 0xA10;
    if const_off + 4 <= iwram.len() {
        let val = u32::from_le_bytes([iwram[const_off], iwram[const_off+1], iwram[const_off+2], iwram[const_off+3]]);
        println!("\nConstant at 0x03000A10: {:#010X} (binary: {:032b})", val, val);
    }

    // Check 0x03000A18 (used by LDR R2, [PC, #80] at 0x030009C0)
    // PC = 0x030009C0 + 8 = 0x030009C8. 0x030009C8 + 80 = 0x03000A18
    let off2 = 0xA18;
    if off2 + 4 <= iwram.len() {
        let val = u32::from_le_bytes([iwram[off2], iwram[off2+1], iwram[off2+2], iwram[off2+3]]);
        println!("Value at 0x03000A18: {:#010X} (binary: {:032b})", val, val);
    }

    // Dump more handler code (after 0x030009F4)
    println!("\n=== Handler code continued from 0x030009F8 ===");
    for i in 0..12 {
        let off = 0x9F8 + i * 4;
        if off + 4 <= iwram.len() {
            let word = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
            println!("  {:#010X}: {:#010X}", 0x03000000 + off, word);
        }
    }
}
