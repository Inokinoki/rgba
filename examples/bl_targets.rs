use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom().to_vec();

    // Decode BL targets from the loading function
    let bl_pairs: Vec<(u32, u32)> = vec![
        (0x080D0AB8, 0x080D0ABA),
        (0x080D0AC4, 0x080D0AC6),
        (0x080D0ACE, 0x080D0AD0),
        (0x080D0AD8, 0x080D0ADA),
        (0x080D0AE2, 0x080D0AE4),
        (0x080D0AEA, 0x080D0AEC),
        (0x080D0AF8, 0x080D0AFA),
        (0x080D0AFE, 0x080D0B00),
        (0x080D0B0A, 0x080D0B0C),
        (0x080D0B12, 0x080D0B14),
        (0x080D0B1E, 0x080D0B20),
        (0x080D0B24, 0x080D0B26),
        (0x080D0B30, 0x080D0B32),
        (0x080D0B38, 0x080D0B3A),
        (0x080D0B44, 0x080D0B46),
        (0x080D0B4A, 0x080D0B4C),
        (0x080D0B56, 0x080D0B58),
        (0x080D0B5E, 0x080D0B60),
        (0x080D0B6A, 0x080D0B6C),
        (0x080D0B72, 0x080D0B74),
        (0x080D0B7E, 0x080D0B80),
        (0x080D0B84, 0x080D0B86),
        (0x080D0B90, 0x080D0B92),
        (0x080D0B98, 0x080D0B9A),
        (0x080D0BA4, 0x080D0BA6),
        (0x080D0BAA, 0x080D0BAC),
        (0x080D0BB6, 0x080D0BB8),
        (0x080D0BBE, 0x080D0BC0),
        (0x080D0BCA, 0x080D0BCC),
        (0x080D0BD2, 0x080D0BD4),
        (0x080D0BDE, 0x080D0BE0),
        (0x080D0BE6, 0x080D0BE8),
        (0x080D0BF2, 0x080D0BF4),
        (0x080D0BF8, 0x080D0BFA),
    ];

    println!("=== BL targets from loading function ===");
    for (addr1, addr2) in &bl_pairs {
        let off1 = (addr1 - 0x08000000) as usize;
        let off2 = (addr2 - 0x08000000) as usize;
        if off1 + 2 <= rom.len() && off2 + 2 <= rom.len() {
            let hw1 = u16::from_le_bytes([rom[off1], rom[off1 + 1]]);
            let hw2 = u16::from_le_bytes([rom[off2], rom[off2 + 1]]);

            let s = (hw1 >> 10) & 1;
            let imm10 = (hw1 & 0x3FF) as i32;
            let j1 = (hw2 >> 13) & 1;
            let j2 = (hw2 >> 11) & 1;
            let imm11 = (hw2 & 0x7FF) as i32;
            let is_blx = ((hw2 >> 12) & 1) == 0;

            let i1 = 1 - ((j1 as i32) ^ (s as i32));
            let i2 = 1 - ((j2 as i32) ^ (s as i32));

            let mut offset = (s as i32) << 24 | i1 << 23 | i2 << 22 | imm10 << 12 | imm11 << 1;
            if s == 1 {
                offset = offset | -33554432i32;
            }

            let target = (*addr1 as i32 + 4 + offset) as u32;
            let instr_type = if is_blx { "BLX" } else { "BL" };
            println!(
                "{}{:08X}: {:04X} {:04X} → {} {:#010X}",
                if target > 0 { " " } else { "!" },
                addr1,
                hw1,
                hw2,
                instr_type,
                target
            );
        }
    }

    // Also decode the literal pool entries
    println!("\n=== Literal pool at 0x080D0CB0 ===");
    for i in 0..20u32 {
        let off = 0x0CB0 + (i as usize) * 4;
        if off + 4 <= rom.len() {
            let val = u32::from_le_bytes([rom[off], rom[off + 1], rom[off + 2], rom[off + 3]]);
            let addr = 0x080D0000 + off as u32;
            if val >= 0x08000000 && val < 0x0A000000 {
                println!("{:08X}: {:08X} (ROM+{:#X})", addr, val, val - 0x08000000);
            } else if val >= 0x06000000 && val < 0x06018000 {
                println!("{:08X}: {:08X} (VRAM+{:#X})", addr, val, val - 0x06000000);
            } else if val >= 0x02000000 && val < 0x03000000 {
                println!("{:08X}: {:08X} (EWRAM+{:#X})", addr, val, val - 0x02000000);
            } else {
                println!("{:08X}: {:08X}", addr, val);
            }
        }
    }

    println!("\n=== Key sub-function analysis ===");
    println!("Check if the decompress function at BL targets handles LZ77/Huffman correctly.");

    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..200u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let vram = gba.mem().vram();
    println!("\n=== Tile data sample ===");
    for tile in [0, 1, 50, 100, 112, 113] {
        let base = tile * 32;
        print!("Tile {}: ", tile);
        for b in 0..32 {
            if vram[base + b] != 0 {
                print!("{:02X}", vram[base + b]);
            } else {
                print!("..");
            }
        }
        println!();
    }
}
