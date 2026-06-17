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
    let handler_base = 0x03000958usize;
    let handler_off = handler_base - 0x03000000;

    println!("=== ARM disassembly of IRQ handler at 0x03000958 ===");
    for i in 0..40 {
        let off = handler_off + i * 4;
        if off + 4 <= iwram.len() {
            let word = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
            let addr = handler_base + i * 4;
            println!("  {:#010X}: {:#010X}  ; {}", addr, word, disasm_arm(word, addr as u32));
        }
    }

    // Also check: what callbacks are stored near the handler?
    // The handler likely uses a function pointer table
    println!("\n=== IWRAM data around 0x03000A50-0x03000AB0 ===");
    for i in 0..24 {
        let off = 0xA50 - 0x00 + i * 4;
        if off + 4 <= iwram.len() {
            let word = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
            let addr = 0x03000000 + off;
            println!("  {:#010X}: {:#010X}", addr, word);
        }
    }

    // Check IE/IF state right before and after the handler runs
    println!("\n=== IE/IF around VBlank in frame 8 ===");
    // Re-run with state checks
    let mut gba2 = Gba::new();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb2 = vec![0u32; 240 * 160];
    for _ in 0..7 {
        gba2.run_frame_parallel(&mut fb2);
    }

    // Check IE/IF at start of frame 8
    let ie_before = gba2.mem().interrupt.read_register(0x200);
    let if_before = gba2.mem().interrupt.read_register(0x002);
    println!("Before frame 8: IE={:#06X} IF={:#06X}", ie_before, if_before);

    // Run one scanline at a time and check IE/IF
    for sl in 0..228 {
        gba2.run_scanline();
        if sl >= 158 && sl <= 165 {
            let ie = gba2.mem().interrupt.read_register(0x200);
            let irf = gba2.mem().interrupt.read_register(0x002);
            let ime = gba2.mem().interrupt.read_register(0x208);
            let halted = gba2.cpu().is_halted();
            let pc = gba2.cpu().get_instruction_pc();
            let mode = gba2.cpu().get_mode();
            let vcount = gba2.ppu().get_vcount();
            println!("  SL{}: IE={:#06X} IF={:#06X} IME={} halted={} PC={:#010X} mode={:?} vcount={}",
                sl, ie, irf, ime, halted, pc, mode, vcount);
        }
    }

    let ie_after = gba2.mem().interrupt.read_register(0x200);
    let if_after = gba2.mem().interrupt.read_register(0x002);
    let vbctr = u32::from_le_bytes([
        gba2.mem().iwram()[0x7FF8], gba2.mem().iwram()[0x7FF9],
        gba2.mem().iwram()[0x7FFA], gba2.mem().iwram()[0x7FFB]
    ]);
    println!("After frame 8: IE={:#06X} IF={:#06X} VBlankCtr={}", ie_after, if_after, vbctr);
}

fn disasm_arm(word: u32, addr: u32) -> String {
    let cond = (word >> 28) & 0xF;
    let cond_str = match cond {
        0xE => "", 0x0 => "EQ", 0x1 => "NE", 0x2 => "CS", 0x3 => "CC",
        0xA => "GE", 0xB => "LT", 0xC => "GT", 0xD => "LE",
        _ => "?",
    };

    // BX
    if (word & 0x0FFFFFF0) == 0x012FFF10 {
        let rn = word & 0xF;
        return format!("BX{} R{}", cond_str, rn);
    }

    // MRS
    if (word & 0x0FBF0FFF) == 0x010F0000 {
        let rd = (word >> 12) & 0xF;
        return format!("MRS{} R{}, CPSR", cond_str, rd);
    }
    if (word & 0x0FBF0FFF) == 0x014F0000 {
        let rd = (word >> 12) & 0xF;
        return format!("MRS{} R{}, SPSR", cond_str, rd);
    }

    // MSR
    if (word & 0x0FB0FFF0) == 0x0120F000 {
        let rm = word & 0xF;
        return format!("MSR{} CPSR_fc, R{}", cond_str, rm);
    }
    if (word & 0x0FB0FFF0) == 0x0160F000 {
        let rm = word & 0xF;
        return format!("MSR{} SPSR_fc, R{}", cond_str, rm);
    }
    if (word & 0x0FB0F000) == 0x0128F000 || (word & 0x0FB0F000) == 0x0120F000 {
        let imm = word & 0xFF;
        let rot = (word >> 8) & 0xF;
        let val = imm.rotate_right(rot * 2);
        return format!("MSR{} CPSR_fc, #{:#X}", cond_str, val);
    }

    // LDM/STM
    if ((word >> 25) & 0x7) == 0b100 {
        let is_load = (word >> 20) & 1;
        let rn = (word >> 16) & 0xF;
        let reg_list = word & 0xFFFF;
        let p = (word >> 24) & 1;
        let u = (word >> 23) & 1;
        let w = (word >> 21) & 1;
        let addr_mode = match (p, u) {
            (0, 1) => "IA", (1, 1) => "IB", (0, 0) => "DA", (1, 0) => "DB",
            _ => "?"
        };
        let mut regs = Vec::new();
        for i in 0..16 {
            if reg_list & (1 << i) != 0 {
                regs.push(format!("R{}", i));
            }
        }
        let wb = if w != 0 { "!" } else { "" };
        if is_load != 0 {
            format!("LDM{}{} R{}{}, {{{}}}", cond_str, addr_mode, rn, wb, regs.join(","))
        } else {
            format!("STM{}{} R{}{}, {{{}}}", cond_str, addr_mode, rn, wb, regs.join(","))
        }
    } else if ((word >> 26) & 0x3) == 0b01 {
        // LDR/STR
        let is_load = (word >> 20) & 1;
        let rn = (word >> 16) & 0xF;
        let rd = (word >> 12) & 0xF;
        let is_imm = ((word >> 25) & 1) == 0;
        let b = (word >> 22) & 1;
        let size = if b != 0 { "B" } else { "" };
        if is_imm {
            let offset = word & 0xFFF;
            let u = if (word >> 23) & 1 != 0 { "+" } else { "-" };
            if is_load != 0 {
                format!("LDR{}{} R{}, [R{}, #{}{}]", cond_str, size, rd, rn, u, offset)
            } else {
                format!("STR{}{} R{}, [R{}, #{}{}]", cond_str, size, rd, rn, u, offset)
            }
        } else {
            let rm = word & 0xF;
            if is_load != 0 {
                format!("LDR{}{} R{}, [R{}, R{}]", cond_str, size, rd, rn, rm)
            } else {
                format!("STR{}{} R{}, [R{}, R{}]", cond_str, size, rd, rn, rm)
            }
        }
    } else if ((word >> 26) & 0x3) == 0b00 && ((word >> 4) & 0xF) != 0b1001 {
        // Data processing
        let opcode = (word >> 21) & 0xF;
        let rd = (word >> 12) & 0xF;
        let rn = (word >> 16) & 0xF;
        let s = (word >> 20) & 1;
        let s_str = if s != 0 { "S" } else { "" };
        let op_str = match opcode {
            0xD => "MOV", 0xA => "CMP", 0xB => "CMN", 0xC => "ORR",
            0x4 => "ADD", 0x2 => "SUB", 0x0 => "AND", 0x1 => "EOR",
            0x3 => "RSB", 0x5 => "ADC", 0x6 => "SBC", 0x7 => "RSC",
            0xE => "BIC", 0xF => "MVN", _ => "?"
        };
        if (word >> 25) & 1 != 0 {
            // Immediate
            let imm = word & 0xFF;
            let rot = (word >> 8) & 0xF;
            let val = imm.rotate_right(rot * 2);
            if opcode == 0xD || opcode == 0xF {
                format!("{}{}{} R{}, #{:#X}", op_str, cond_str, s_str, rd, val)
            } else if opcode == 0xA || opcode == 0xB {
                format!("{}{} R{}, #{:#X}", op_str, cond_str, rn, val)
            } else {
                format!("{}{}{} R{}, R{}, #{:#X}", op_str, cond_str, s_str, rd, rn, val)
            }
        } else {
            let rm = word & 0xF;
            if opcode == 0xD || opcode == 0xF {
                format!("{}{}{} R{}, R{}", op_str, cond_str, s_str, rd, rm)
            } else if opcode == 0xA || opcode == 0xB {
                format!("{}{} R{}, R{}", op_str, cond_str, rn, rm)
            } else {
                format!("{}{}{} R{}, R{}, R{}", op_str, cond_str, s_str, rd, rn, rm)
            }
        }
    } else if ((word >> 25) & 0x7) == 0b101 {
        // Branch
        let offset = (word & 0xFFFFFF) as u32;
        let offset = if offset & 0x800000 != 0 { (offset | 0xFF000000) as i32 } else { offset as i32 };
        let target = addr.wrapping_add(8).wrapping_add((offset << 2) as u32);
        let link = if (word >> 24) & 1 != 0 { "L" } else { "" };
        format!("B{}{} {:#010X}", link, cond_str, target)
    } else {
        format!("?")
    }
}
