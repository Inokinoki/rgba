use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..195 { gba.run_frame_parallel(&mut fb); }

    // Dump THUMB code at 0x0805E6E0-0x0805E780
    println!("=== THUMB code at 0x0805E6E0-0x0805E780 ===");
    for addr in (0x0805E6E0..0x0805E780).step_by(2) {
        let half = gba.mem_mut().read_half(addr);
        println!("0x{:08X}: 0x{:04X}  {}", addr, half, disasm_thumb(half, addr));
    }
    
    // Also dump what calls the IntrWait wrapper
    // The IntrWait wrapper is at 0x080D2F0C (MOVS R2,#0; SWI 4; BX LR)
    // It's called via BL. Let's look at the code around the main loop
    println!("\n=== THUMB code at 0x080D2F00-0x080D2F20 (IntrWait wrapper) ===");
    for addr in (0x080D2F00..0x080D2F20).step_by(2) {
        let half = gba.mem_mut().read_half(addr);
        println!("0x{:08X}: 0x{:04X}  {}", addr, half, disasm_thumb(half, addr));
    }
    
    // Check 0x080D3000 region (552 hits)
    println!("\n=== THUMB code at 0x080D3000-0x080D3080 ===");
    for addr in (0x080D3000..0x080D3080).step_by(2) {
        let half = gba.mem_mut().read_half(addr);
        println!("0x{:08X}: 0x{:04X}  {}", addr, half, disasm_thumb(half, addr));
    }
}

fn disasm_thumb(op: u16, pc: u32) -> String {
    if op >> 13 == 0 {
        let op_type = (op >> 11) & 3;
        let offset = (op >> 6) & 0x1F;
        let rs = (op >> 3) & 7;
        let rd = op & 7;
        match op_type {
            0 => return if offset==0 {format!("MOV R{},R{}", rd, rs)} else {format!("LSL R{},R{},#{}", rd, rs, offset)},
            1 => return format!("LSR R{},R{},#{}", rd, rs, if offset==0 {32} else {offset}),
            2 => return format!("ASR R{},R{},#{}", rd, rs, if offset==0 {32} else {offset}),
            _ => {}
        }
    }
    if op >> 10 == 0b00011 {
        let is_imm = op & (1<<10) == 0;
        let is_sub = op & (1<<9) != 0;
        let rn_off = (op >> 6) & 7;
        let rs = (op >> 3) & 7;
        let rd = op & 7;
        let src = if is_imm { format!("#{}", rn_off) } else { format!("R{}", rn_off) };
        return format!("{} R{},R{},{}", if is_sub {"SUBS"} else {"ADDS"}, rd, rs, src);
    }
    if op >> 13 == 0b001 {
        let op_type = (op >> 11) & 3;
        let rd = (op >> 8) & 7;
        let imm = op & 0xFF;
        match op_type {
            0 => return format!("MOVS R{},#0x{:02X}", rd, imm),
            1 => return format!("CMP R{},#0x{:02X}", rd, imm),
            2 => return format!("ADDS R{},#0x{:02X}", rd, imm),
            3 => return format!("SUBS R{},#0x{:02X}", rd, imm),
            _ => {}
        }
    }
    if op >> 10 == 0b010000 {
        let op_type = (op >> 6) & 0xF;
        let rs = (op >> 3) & 7;
        let rd = op & 7;
        let name = match op_type {
            0=>"ANDS",1=>"EORS",2=>"LSLS",3=>"LSRS",4=>"ASRS",5=>"ADCS",6=>"SBCS",
            7=>"RORS",8=>"TST",9=>"NEGS",10=>"CMP",11=>"CMN",12=>"ORRS",
            13=>"MULS",14=>"BICS",15=>"MVNS",_=>"?"
        };
        return format!("{} R{},R{}", name, rd, rs);
    }
    if op >> 10 == 0b010001 {
        let op_type = (op >> 8) & 3;
        let h1 = (op >> 7) & 1;
        let h2 = (op >> 6) & 1;
        let rs = ((h2 << 3) | ((op >> 3) & 7)) as usize;
        let rd = ((h1 << 3) | (op & 7)) as usize;
        match op_type {
            0 => return format!("ADD R{},R{}", rd, rs),
            1 => return format!("CMP R{},R{}", rd, rs),
            2 => return format!("MOV R{},R{}", rd, rs),
            3 => return if rd == 15 { format!("BX R{}", rs) } else { format!("BLX R{}", rs) },
            _ => {}
        }
    }
    if op >> 11 == 0b01001 {
        let rd = (op >> 8) & 7;
        let word8 = (op & 0xFF) as u32;
        return format!("LDR R{},[PC,#0x{:02X}]", rd, word8*4);
    }
    if op >> 12 == 0b0101 {
        let ro = (op >> 6) & 7;
        let rb = (op >> 3) & 7;
        let rd = op & 7;
        let load = op & (1<<11) != 0;
        let byte = op & (1<<10) != 0;
        if !load && !byte { return format!("STR R{},[R{},R{}]", rd, rb, ro); }
        if !load && byte { return format!("STRB R{},[R{},R{}]", rd, rb, ro); }
        if load && !byte { return format!("LDR R{},[R{},R{}]", rd, rb, ro); }
        return format!("LDRB R{},[R{},R{}]", rd, rb, ro);
    }
    if op >> 13 == 0b011 {
        let b = op & (1<<12) == 0;
        let load = op & (1<<11) != 0;
        let offset5 = ((op >> 6) & 0x1F) as u32;
        let rb = (op >> 3) & 7;
        let rd = op & 7;
        if b {
            let name = if load {"LDRB"} else {"STRB"};
            return format!("{} R{},[R{},#0x{:02X}]", name, rd, rb, offset5);
        } else {
            let name = if load {"LDR"} else {"STR"};
            return format!("{} R{},[R{},#0x{:02X}]", name, rd, rb, offset5*4);
        }
    }
    if op >> 12 == 0b1000 {
        let load = op & (1<<11) != 0;
        let offset5 = ((op >> 6) & 0x1F) as u32;
        let rb = (op >> 3) & 7;
        let rd = op & 7;
        return format!("{} R{},[R{},#0x{:02X}]", if load {"LDRH"} else {"STRH"}, rd, rb, offset5*2);
    }
    if op >> 12 == 0b1001 {
        let load = op & (1<<11) != 0;
        let rd = (op >> 8) & 7;
        let word8 = (op & 0xFF) as u32;
        return format!("{} R{},[SP,#0x{:02X}]", if load {"LDR"} else {"STR"}, rd, word8*4);
    }
    if op >> 11 == 0b10101 {
        let rd = (op >> 8) & 7;
        let word8 = (op & 0xFF) as u32;
        return format!("ADD R{},SP,#0x{:02X}", rd, word8*4);
    }
    if op >> 8 == 0b10110000 {
        let s = op & (1<<7) != 0;
        let imm = (op & 0x7F) as u32 * 4;
        return format!("{} SP,#0x{:02X}", if s {"SUB"} else {"ADD"}, imm);
    }
    if op >> 9 == 0b1011 && ((op >> 8) & 1) == 0 {
        let l = op & (1<<11) != 0;
        let r = op & (1<<8) != 0;
        let mut regs = Vec::new();
        for i in 0..8 { if op & (1<<i) != 0 { regs.push(format!("R{}", i)); } }
        let name = if l {"POP"} else {"PUSH"};
        let extra = if r { if l {",PC"} else {",LR"} } else { "" };
        return format!("{} {{{}{}}}", name, regs.join(","), extra);
    }
    if op >> 12 == 0b1100 {
        let load = op & (1<<11) != 0;
        let rb = (op >> 8) & 7;
        let mut regs = Vec::new();
        for i in 0..8 { if op & (1<<i) != 0 { regs.push(format!("R{}", i)); } }
        return format!("{} R{}!,{{{}}}", if load {"LDMIA"} else {"STMIA"}, rb, regs.join(","));
    }
    if op >> 12 == 0b1101 {
        let cond = (op >> 8) & 0xF;
        if cond == 0xF { return format!("SWI #0x{:02X}", op & 0xFF); }
        if cond < 0xE {
            let offset = (op & 0xFF) as i8 as i32;
            let target = (pc as i32 + 4 + offset * 2) as u32;
            let cn = ["EQ","NE","CS","CC","MI","PL","VS","VC","HI","LS","GE","LT","GT","LE"][cond as usize];
            return format!("B{} 0x{:08X}", cn, target);
        }
        return "UDEF".to_string();
    }
    if op >> 11 == 0b11100 {
        let offset = (op & 0x7FF) as u32;
        let offset = if offset & 0x400 != 0 { (offset | 0xFFFFF800u32) as i32 } else { offset as i32 };
        let target = (pc as i32 + 4 + offset * 2) as u32;
        return format!("B 0x{:08X}", target);
    }
    if op >> 11 == 0b11110 || op >> 11 == 0b11111 {
        return format!("BL part 0x{:04X}", op);
    }
    format!(".hword 0x{:04X}", op)
}
