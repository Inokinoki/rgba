use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom().to_vec();
    let mut fb = vec![0u32; 240 * 160];
    
    // Disassemble key code regions as THUMB
    let regions: &[(u32, usize, &str)] = &[
        (0x080D7098, 20, "VBlank callback"),
        (0x080D2F08, 10, "IntrWait wrapper area"),
        (0x080D30B8, 20, "Jump table"),
        (0x080D1640, 16, "Loop at D1656"),
        (0x080D13F0, 32, "Loop at D13FE"),
        (0x08009910, 16, "Loop at 0991C"),
        (0x080D25C0, 32, "Loop at D25F6"),
    ];
    
    for &(base, count, name) in regions {
        println!("\n=== {} (0x{:08X}) ===", name, base);
        for i in 0..count {
            let addr = base + (i as u32) * 2;
            let off = (addr - 0x08000000) as usize;
            if off + 1 >= rom.len() { break; }
            let half = u16::from_le_bytes([rom[off], rom[off+1]]);
            print!("0x{:08X}: 0x{:04X}  ", addr, half);
            
            // Simple THUMB decode
            if half >> 12 == 0xD && ((half >> 8) & 0xF) < 0xE {
                let off = (half & 0xFF) as i8 as i32;
                let tgt = (addr as i32 + 4 + off * 2) as u32;
                let cn = ["EQ","NE","CS","CC","MI","PL","VS","VC","HI","LS","GE","LT","GT","LE"];
                println!("B{} 0x{:08X}", cn[((half>>8)&0xF) as usize], tgt);
            } else if half >> 11 == 0b11100 {
                let off = (half & 0x7FF);
                let off = if off & 0x400 != 0 { (off | 0xF800) as i16 as i32 } else { off as i32 };
                println!("B 0x{:08X}", (addr as i32+4+off*2) as u32);
            } else if half >> 11 == 0b11110 {
                println!("BL prefix");
            } else if half >> 11 == 0b01001 {
                println!("LDR R{}, [PC, #{}]", (half>>8)&7, (half&0xFF)*4);
            } else if half >> 13 == 0b001 && (half >> 11) & 3 != 3 {
                let op = (half>>11)&3; let rd=(half>>8)&7; let imm=half&0xFF;
                println!("{} R{}, #{}", ["MOVS","CMP","ADDS","SUBS"][op as usize], rd, imm);
            } else if half >> 12 == 0b0101 {
                let op = (half>>9)&7;
                let ro = (half>>6)&7; let rb = (half>>3)&7; let rd = half&7;
                let ops = ["STR","STRH","STRB","LDRSB","LDR","LDRH","LDRB","LDRSH"];
                println!("{} R{}, [R{}, R{}]", ops[op as usize], rd, rb, ro);
            } else if half >> 10 == 0b010000 {
                let op = (half>>6)&0xF; let rs=(half>>3)&7; let rd=half&7;
                let ops = ["AND","EOR","LSL","LSR","ASR","ADC","SBC","ROR","TST","NEG","CMP","CMN","ORR","MUL","BIC","MVN"];
                println!("{} R{}, R{}", ops[op as usize], rd, rs);
            } else if (half >> 10) == 0b001100 {
                let op = (half>>8)&3; let off = (half&0xFF) as u32;
                let ops = ["ADD","SUB","MOV","CMP"];
                println!("{} SP, #{}", ops[op as usize], off*4);
            } else if (half >> 7) == 0b101100000 {
                println!("ADD SP, #{}", (half & 0x7F) * 4);
            } else if (half >> 7) == 0b101100001 {
                println!("SUB SP, #{}", (half & 0x7F) * 4);
            } else if half >> 8 == 0b10111111 {
                // SWI
                println!("SWI {}", half & 0xFF);
            } else if half >> 7 == 0b010000000 {
                let regs = half & 0xFF;
                print!("PUSH {{");
                let mut first = true;
                for i in 0..8 { if regs & (1<<i) != 0 { if !first { print!(","); } print!("R{}", i); first = false; } }
                if half & 0x100 != 0 { if !first { print!(","); } print!("LR"); }
                println!("}}");
            } else if half >> 8 == 0b10111100 {
                let regs = half & 0xFF;
                print!("POP {{");
                let mut first = true;
                for i in 0..8 { if regs & (1<<i) != 0 { if !first { print!(","); } print!("R{}", i); first = false; } }
                if half & 0x100 != 0 { if !first { print!(","); } print!("PC"); }
                println!("}}");
            } else if half >> 12 == 0b0101 && ((half>>9)&7) < 4 {
                let op = (half>>9)&7;
                let off = half & 0x1F;
                println!("STR/LDR off={}", off);
            } else if half >> 11 == 0b01001 {
                let rd = (half>>8)&7; let off = ((half&0xFF) as u32)*4;
                let pc = ((addr + 4) & !3u32) as u32;
                println!("LDR R{}, [PC, #{}] = [0x{:08X}]", rd, off, pc + off);
            } else if half == 0x46C0 {
                println!("NOP");
            } else if half >> 7 == 0b010001100 {
                let rm = (half>>3)&0xF; let rd = (half&7) | ((half>>7)&8);
                println!("MOV R{}, R{}", rd, rm);
            } else if half >> 7 == 0b010001110 {
                let rm = (half>>3)&0xF; let rd = (half&7) | ((half>>7)&8);
                println!("BX R{}", rm);
            } else if half >> 8 == 0x47 {
                let rm = (half>>3)&0xF;
                println!("BX R{}", rm);
            } else {
                println!(".hword");
            }
        }
    }
    
    // Check specific memory values the game might be polling
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }
    
    println!("\n=== Game state at frame 7 (stuck) ===");
    let iwram = gba.mem().iwram();
    
    // Check common state variable locations
    for offset in [0x100usize, 0x104, 0x108, 0x10C, 0x110, 0x114, 0x118, 0x11C,
                    0x120, 0x124, 0x128, 0x12C, 0x130, 0x134, 0x138, 0x13C,
                    0x140, 0x144, 0x148, 0x14C, 0x150, 0x154, 0x158, 0x15C,
                    0xFF8, 0xFFC, 0x7FF8, 0x7FFC] {
        let val = u32::from_le_bytes([iwram[offset], iwram[offset+1],
            iwram[offset+2], iwram[offset+3]]);
        if val != 0 {
            println!("  IWRAM[0x{:04X}] = 0x{:08X}", offset, val);
        }
    }
}
