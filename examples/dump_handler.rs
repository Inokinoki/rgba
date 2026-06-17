use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run 3 frames to get past boot
    for _ in 0..3 {
        gba.run_frame_parallel(&mut fb);
    }

    let m = gba.mem_mut();

    // Dump handler code at 0x03000958 (ARM mode)
    println!("Game IRQ handler at 0x03000958:");
    for i in 0..40 {
        let addr = 0x03000958 + i * 4;
        let word = m.read_word(addr);
        let cond = (word >> 28) & 0xF;
        let cond_str = match cond {
            0 => "EQ", 1 => "NE", 2 => "CS", 3 => "CC",
            4 => "MI", 5 => "PL", 6 => "VS", 7 => "VC",
            8 => "HI", 9 => "LS", 10 => "GE", 11 => "LT",
            12 => "GT", 13 => "LE", 14 => "AL", _ => "??",
        };

        // Decode common ARM instructions
        let desc = if word & 0x0FF00000 == 0x01200000 {
            // BX
            format!("BX{} R{}", cond_str, word & 0xF)
        } else if word & 0x0E100000 == 0x04100000 {
            // LDR
            let rd = (word >> 12) & 0xF;
            let rn = (word >> 16) & 0xF;
            let off = word & 0xFFF;
            let u = if word & (1 << 23) != 0 { "+" } else { "-" };
            let pre = if word & (1 << 24) != 0 { "!" } else { "" };
            format!("LDR{} R{}, [R{}, #{}{}]{}", cond_str, rd, rn, u, off, pre)
        } else if word & 0x0E100000 == 0x06100000 {
            // LDRB
            let rd = (word >> 12) & 0xF;
            let rn = (word >> 16) & 0xF;
            let off = word & 0xFFF;
            let u = if word & (1 << 23) != 0 { "+" } else { "-" };
            format!("LDRB{} R{}, [R{}, #{}{}]", cond_str, rd, rn, u, off)
        } else if word & 0x0FFFFFF0 == 0x012FFF10 {
            let rm = word & 0xF;
            format!("BX{} R{}", cond_str, rm)
        } else if word & 0x0E200000 == 0x02800000 {
            // ADD imm
            let rd = (word >> 12) & 0xF;
            let rn = (word >> 16) & 0xF;
            let rot = (word >> 8) & 0xF;
            let imm = word & 0xFF;
            let val = imm.rotate_right(rot * 2);
            format!("ADD{} R{}, R{}, #0x{:08X}", cond_str, rd, rn, val)
        } else if word & 0x0E200000 == 0x02000000 {
            // AND imm, etc
            let rd = (word >> 12) & 0xF;
            let rn = (word >> 16) & 0xF;
            let op = (word >> 21) & 0xF;
            let rot = (word >> 8) & 0xF;
            let imm = word & 0xFF;
            let val = imm.rotate_right(rot * 2);
            let mnem = match op {
                0 => "AND", 1 => "EOR", 2 => "SUB", 3 => "RSB",
                4 => "ADD", 5 => "ADC", 6 => "SBC", 7 => "RSC",
                8 => "TST", 9 => "TEQ", 10 => "CMP", 11 => "CMN",
                12 => "ORR", 13 => "MOV", 14 => "BIC", 15 => "MVN",
                _ => "???"
            };
            let s = if word & (1 << 20) != 0 { "S" } else { "" };
            if op == 13 || op == 15 {
                format!("{}{}{} R{}, #0x{:08X}", mnem, cond_str, s, rd, val)
            } else if op >= 8 && op <= 11 {
                format!("{}{} R{}, #0x{:08X}", mnem, cond_str, rn, val)
            } else {
                format!("{}{}{} R{}, R{}, #0x{:08X}", mnem, cond_str, s, rd, rn, val)
            }
        } else if word & 0x0E000000 == 0x08000000 {
            // LDM/STM
            let rn = (word >> 16) & 0xF;
            let reg_list = word & 0xFFFF;
            let load = word & (1 << 20) != 0;
            let writeback = word & (1 << 21) != 0;
            let pre = word & (1 << 24) != 0;
            let up = word & (1 << 23) != 0;

            let mut regs_str = String::new();
            for r in 0..16 {
                if reg_list & (1 << r) != 0 {
                    if !regs_str.is_empty() { regs_str.push_str(","); }
                    if r == 14 { regs_str.push_str("LR"); }
                    else if r == 15 { regs_str.push_str("PC"); }
                    else if r == 13 { regs_str.push_str("SP"); }
                    else { regs_str.push_str(&format!("R{}", r)); }
                }
            }
            let mnem = if load {
                if pre && up { "LDMIB" } else if !pre && up { "LDMIA" }
                else if pre && !up { "LDMDB" } else { "LDMDA" }
            } else {
                if pre && up { "STMIB" } else if !pre && up { "STMIA" }
                else if pre && !up { "STMDB" } else { "STMDA" }
            };
            let wb = if writeback { "!" } else { "" };
            format!("{}{} R{}{}, {{{}}}", mnem, cond_str, rn, wb, regs_str)
        } else if word & 0x0E000000 == 0x0A000000 {
            // Branch
            let link = word & (1 << 24) != 0;
            let off = (word & 0xFFFFFF) as i32;
            let off = if off & 0x800000 != 0 { off | 0xFF000000u32 as i32 } else { off };
            let target = (addr as i32 + 8 + off * 4) as u32;
            let mnem = if link { "BL" } else { "B" };
            format!("{}{} 0x{:08X}", mnem, cond_str, target)
        } else if word & 0x0F100000 == 0x01000000 {
            // MRS/MSR
            if word & 0x00400000 != 0 {
                let rd = (word >> 12) & 0xF;
                format!("MRS{} R{}, SPSR", cond_str, rd)
            } else {
                let rd = (word >> 12) & 0xF;
                format!("MRS{} R{}, CPSR", cond_str, rd)
            }
        } else if word & 0x0FB0FFF0 == 0x0128F000 {
            format!("MSR{} SPSR, R{}", cond_str, word & 0xF)
        } else if word & 0x0FB0FFF0 == 0x0129F000 {
            format!("MSR{} CPSR, R{}", cond_str, word & 0xF)
        } else if word & 0x0FB000F0 == 0x01200000 {
            let mask = if word & 0xF0000 != 0 { "_fc" } else { "" };
            format!("MSR{} CPSR{}, R{}", cond_str, mask, word & 0xF)
        } else if word & 0x0E000000 == 0x00000000 {
            // Data processing register
            let rd = (word >> 12) & 0xF;
            let rn = (word >> 16) & 0xF;
            let op = (word >> 21) & 0xF;
            let rm = word & 0xF;
            let mnem = match op {
                0 => "AND", 1 => "EOR", 2 => "SUB", 3 => "RSB",
                4 => "ADD", 5 => "ADC", 6 => "SBC", 7 => "RSC",
                8 => "TST", 9 => "TEQ", 10 => "CMP", 11 => "CMN",
                12 => "ORR", 13 => "MOV", 14 => "BIC", 15 => "MVN",
                _ => "???"
            };
            let s = if word & (1 << 20) != 0 { "S" } else { "" };
            if op == 13 || op == 15 {
                format!("{}{}{} R{}, R{}", mnem, cond_str, s, rd, rm)
            } else if op >= 8 && op <= 11 {
                format!("{}{} R{}, R{}", mnem, cond_str, rn, rm)
            } else {
                format!("{}{}{} R{}, R{}, R{}", mnem, cond_str, s, rd, rn, rm)
            }
        } else {
            format!("???(0x{:08X})", word)
        };

        println!("  0x{:08X}: 0x{:08X}  {}", addr, word, desc);
    }
}
