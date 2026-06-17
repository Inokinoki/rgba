use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }

    // Dump ARM code from 0x080D0900 to 0x080D0980
    println!("=== ARM code at 0x080D0900-0x080D0980 ===");
    for addr in (0x080D0900..0x080D0980).step_by(4) {
        let word = gba.mem_mut().read_word_fast(addr);
        println!("0x{:08X}: 0x{:08X}  {}", addr, word, disasm_arm(word, addr));
    }

    // Also dump 0x080D0BF0-0x080D0C80
    println!("\n=== ARM code at 0x080D0BF0-0x080D0C80 ===");
    for addr in (0x080D0BF0..0x080D0C80).step_by(4) {
        let word = gba.mem_mut().read_word_fast(addr);
        println!("0x{:08X}: 0x{:08X}  {}", addr, word, disasm_arm(word, addr));
    }
}

fn disasm_arm(op: u32, _pc: u32) -> String {
    // Basic ARM disassembler for common instructions
    let cond = match (op >> 28) & 0xF {
        0 => "EQ", 1 => "NE", 2 => "CS", 3 => "CC",
        4 => "MI", 5 => "PL", 6 => "VS", 7 => "VC",
        8 => "HI", 9 => "LS", 10 => "GE", 11 => "LT",
        12 => "GT", 13 => "LE", 14 => "", 15 => "??",
        _ => "?",
    };

    // Branch
    if (op >> 25) & 7 == 0b101 {
        let offset = (op & 0xFFFFFF) as u32;
        let offset = if offset & 0x800000 != 0 { (offset | 0xFF000000u32) as i32 } else { offset as i32 };
        let target = (_pc.wrapping_add(8) as i32 + offset * 4) as u32;
        let link = if op & 0x1000000 != 0 { "L" } else { "" };
        return format!("B{}{} 0x{:08X}", link, cond, target);
    }

    // LDR/STR
    if (op >> 26) & 3 == 0b01 {
        let load = op & (1 << 20) != 0;
        let writeback = op & (1 << 21) != 0;
        let byte = op & (1 << 22) != 0;
        let up = op & (1 << 23) != 0;
        let pre = op & (1 << 24) != 0;
        let rn = (op >> 16) & 0xF;
        let rd = (op >> 12) & 0xF;
        
        let offset = if op & (1 << 25) != 0 {
            // Register offset
            let rm = op & 0xF;
            let shift = (op >> 7) & 0x1F;
            if shift == 0 { format!("R{}", rm) }
            else { format!("R{},LSL#{}", rm, shift) }
        } else {
            format!("#{}", op & 0xFFF)
        };
        
        let sign = if up { "+" } else { "-" };
        let addr_mode = if pre {
            if writeback { format!("[R{},{}{}]!", rn, sign, offset) }
            else { format!("[R{},{}{}]", rn, sign, offset) }
        } else {
            format!("[R{}],{}{}", rn, sign, offset)
        };
        
        let op_name = if load { "LDR" } else { "STR" };
        let b = if byte { "B" } else { "" };
        return format!("{}{}{} R{},{}", op_name, b, cond, rd, addr_mode);
    }

    // LDM/STM
    if (op >> 25) & 7 == 0b100 {
        let load = op & (1 << 20) != 0;
        let writeback = op & (1 << 21) != 0;
        let user = op & (1 << 22) != 0;
        let up = op & (1 << 23) != 0;
        let pre = op & (1 << 24) != 0;
        let rn = (op >> 16) & 0xF;
        let reglist = op & 0xFFFF;
        
        let mut regs = Vec::new();
        for i in 0..16 {
            if reglist & (1 << i) != 0 { regs.push(format!("R{}", i)); }
        }
        
        let mode = match (pre, up) {
            (true, true) => "IA!", (true, false) => "DB!",
            (false, true) => "IA", (false, false) => "DB",
        };
        let wb = if writeback { "!" } else { "" };
        let op_name = if load { "LDM" } else { "STM" };
        return format!("{}{} R{}{} {{{}}}", op_name, cond, rn, wb, regs.join(","));
    }

    // Data processing
    if (op >> 26) & 3 == 0b00 {
        let opcode = (op >> 21) & 0xF;
        let s = op & (1 << 20) != 0;
        let rn = (op >> 16) & 0xF;
        let rd = (op >> 12) & 0xF;
        
        let op_name = match opcode {
            0 => "AND", 1 => "EOR", 2 => "SUB", 3 => "RSB",
            4 => "ADD", 5 => "ADC", 6 => "SBC", 7 => "RSC",
            8 => "TST", 9 => "TEQ", 10 => "CMP", 11 => "CMN",
            12 => "ORR", 13 => "MOV", 14 => "BIC", 15 => "MVN",
            _ => "?",
        };
        
        let op2 = if op & (1 << 25) != 0 {
            // Immediate
            let imm = op & 0xFF;
            let rot = ((op >> 8) & 0xF) * 2;
            format!("#0x{:X}", imm.rotate_right(rot))
        } else {
            let rm = op & 0xF;
            let shift_type = (op >> 5) & 3;
            if op & (1 << 4) != 0 {
                let rs = (op >> 8) & 0xF;
                let st = match shift_type { 0=>"LSL",1=>"LSR",2=>"ASR",3=>"ROR",_=>"?" };
                format!("R{},{} R{}", rm, st, rs)
            } else {
                let shift = (op >> 7) & 0x1F;
                let st = match shift_type {
                    0 => if shift==0 { "".to_string() } else { format!(",LSL#{}", shift) },
                    1 => if shift==0 { ",LSR#32".to_string() } else { format!(",LSR#{}", shift) },
                    2 => if shift==0 { ",ASR#32".to_string() } else { format!(",ASR#{}", shift) },
                    3 => if shift==0 { ",RRX".to_string() } else { format!(",ROR#{}", shift) },
                    _ => "?".to_string(),
                };
                format!("R{}{}", rm, st)
            }
        };
        
        let s_flag = if s && opcode <= 7 { "S" } else { "" };
        
        match opcode {
            13 | 15 => return format!("{}{}{} R{},{}", op_name, cond, s_flag, rd, op2),
            8..=11 => return format!("{}{} R{},{}", op_name, cond, rn, op2),
            _ => return format!("{}{}{} R{},R{},{}", op_name, cond, s_flag, rd, rn, op2),
        }
    }

    // MRS
    if (op & 0x0FBF0FFF) == 0x010F0000 {
        let rd = (op >> 12) & 0xF;
        return format!("MRS{} R{},CPSR", cond, rd);
    }
    if (op & 0x0FBF0FFF) == 0x014F0000 {
        let rd = (op >> 12) & 0xF;
        return format!("MRS{} R{},SPSR", cond, rd);
    }
    
    // MSR
    if (op & 0x0FB0FFF0) == 0x0120F000 {
        let rm = op & 0xF;
        return format!("MSR{} CPSR_fc,R{}", cond, rm);
    }
    if (op & 0x0FB0FFF0) == 0x0160F000 {
        let rm = op & 0xF;
        return format!("MSR{} SPSR_fc,R{}", cond, rm);
    }
    if (op & 0x0FFFFFF0) == 0x0128F000 {
        let imm = op & 0xFF;
        let rot = ((op >> 8) & 0xF) * 2;
        return format!("MSR{} CPSR_fc,#0x{:X}", cond, imm.rotate_right(rot));
    }

    // SWI
    if (op >> 24) & 0xF == 0xF {
        return format!("SWI{} #0x{:X}", cond, op & 0xFFFFFF);
    }

    // BX
    if (op & 0x0FFFFFF0) == 0x012FFF10 {
        let rm = op & 0xF;
        return format!("BX{} R{}", cond, rm);
    }

    format!(".word 0x{:08X}", op)
}
