use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom().to_vec();
    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let vram = gba.mem().vram();

    println!("=== Tile 113 data (last loaded tile) ===");
    let tile113_off = 113 * 32;
    for b in 0..32 {
        print!("{:02X}", vram[tile113_off + b]);
    }
    println!();

    println!("\n=== Tile 0 data ===");
    for b in 0..32 {
        print!("{:02X}", vram[b]);
    }
    println!();

    println!("\n=== Tile 1 data ===");
    for b in 0..32 {
        print!("{:02X}", vram[32 + b]);
    }
    println!();

    let ewram = gba.mem().wram();
    println!("\n=== EWRAM nonzero regions ===");
    let mut regions: Vec<(usize, usize)> = Vec::new();
    let mut start = None;
    let mut count = 0;
    for i in 0..ewram.len() {
        if ewram[i] != 0 {
            if start.is_none() {
                start = Some(i);
            }
            count += 1;
        } else {
            if count > 64 {
                regions.push((start.unwrap(), count));
            }
            start = None;
            count = 0;
        }
    }
    if count > 64 {
        regions.push((start.unwrap(), count));
    }
    for (off, len) in &regions {
        println!("  EWRAM+{:#X}: {} bytes ({:#X})", off, len, len);
    }

    println!("\n=== Loading function BL target analysis ===");
    let bl_calls: Vec<(u32, u32, u16, u16)> = vec![
        (0x080D0AB8, 0x080D0ABA, 0xF7FF, 0xFF4F),
        (0x080D0AC4, 0x080D0AC6, 0xF7FF, 0xFF48),
        (0x080D0AD0, 0x080D0AD2, 0xF7FF, 0xFF43),
        (0x080D0B7A, 0x080D0B7C, 0xF7FF, 0xFEEF),
        (0x080D0B88, 0x080D0B8A, 0xF7FF, 0xFEE8),
        (0x080D0B92, 0x080D0B94, 0xF7FF, 0xFEE2),
        (0x080D0BA0, 0x080D0BA2, 0xF7FF, 0xFEDC),
        (0x080D0BB6, 0x080D0BB8, 0xF7FF, 0xFED1),
        (0x080D0BC8, 0x080D0BCC, 0xF7FF, 0xFEC6),
    ];

    for (addr1, addr2, hw1, hw2) in &bl_calls {
        let off1 = (addr1 - 0x08000000) as usize;
        let off2 = (addr2 - 0x08000000) as usize;
        let actual_hw1 = u16::from_le_bytes([rom[off1], rom[off1 + 1]]);
        let actual_hw2 = u16::from_le_bytes([rom[off2], rom[off2 + 1]]);

        let s = (actual_hw1 >> 10) & 1;
        let imm10 = (actual_hw1 & 0x3FF) as i32;
        let j1 = (actual_hw2 >> 13) & 1;
        let j2 = (actual_hw2 >> 11) & 1;
        let imm11 = (actual_hw2 & 0x7FF) as i32;
        let is_blx = ((actual_hw2 >> 12) & 1) == 0;

        let i1 = 1 - ((j1 as i32) ^ (s as i32));
        let i2 = 1 - ((j2 as i32) ^ (s as i32));

        let mut offset = (s as i32) << 24 | i1 << 23 | i2 << 22 | imm10 << 12 | imm11 << 1;
        if s == 1 {
            offset = offset | -33554432i32;
        }

        let target = (*addr1 as i32 + 4 + offset) as u32;
        let instr_type = if is_blx { "BLX" } else { "BL" };
        println!(
            "  {}{:08X}: {:04X}{:04X} -> {:#010X}",
            instr_type, addr1, actual_hw1, actual_hw2, target
        );
    }

    println!("\n=== Key question: Does the game expect to load tiles after frame 189? ===");
    println!("At frame 195, BG registers:");
    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!("  DISPCNT: {:#06X} (mode {})", dispcnt, dispcnt & 7);
    for bg in 0..4u32 {
        let off = 0x08 + bg as usize * 2;
        let bgcnt = u16::from_le_bytes([io[off], io[off + 1]]);
        let enabled = (dispcnt >> (8 + bg)) & 1;
        if bgcnt != 0 || enabled != 0 {
            let char_base = ((bgcnt >> 2) & 3) as u32 * 0x4000;
            let screen_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
            println!(
                "  BG{}: enabled={} char={:#X} screen={:#X}",
                bg, enabled, char_base, screen_base
            );
        }
    }

    println!("\n=== Check: What is the game's main loop doing? ===");
    let pc = gba.cpu().get_pc();
    let cpsr = gba.cpu().get_cpsr();
    let thumb = (cpsr >> 5) & 1;
    println!(
        "CPU PC: {:#010X} ({} mode)",
        pc,
        if thumb != 0 { "THUMB" } else { "ARM" }
    );

    let rom_off = (pc - 0x08000000) as usize & !1;
    for i in 0..8u32 {
        let off = rom_off + i as usize * 2;
        if off + 2 <= rom.len() {
            let opcode = u16::from_le_bytes([rom[off], rom[off + 1]]);
            println!("  {:08X}: {:04X}", 0x08000000 + off as u32, opcode);
        }
    }
}
