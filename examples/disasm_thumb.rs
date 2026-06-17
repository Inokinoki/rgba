use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }

    // Dump THUMB code at hot addresses
    for (start, end, label) in [
        (0x080D0900u32, 0x080D0978u32, "HOT LOOP"),
        (0x080D0BF0, 0x080D0C80, "SECOND HOT"),
        (0x080D0B10, 0x080D0B80, "THIRD HOT"),
    ] {
        println!("=== THUMB {} 0x{:08X}-0x{:08X} ===", label, start, end);
        for addr in (start..end).step_by(2) {
            let half = gba.mem_mut().read_half(addr);
            println!("0x{:08X}: 0x{:04X}  {}", addr, half, disasm_thumb(half, addr));
        }
        println!();
    }
}

fn disasm_thumb(op: u16, pc: u32) -> String {
    // Format 1: Move shifted register (000oo xxx xxx xxx xxx)
    if op >> 13 == 0 {
        let op_type = (op >> 11) & 3;
        let offset = (op >> 6) & 0x1F;
        let rs = (op >> 3) & 7;
        let rd = op & 7;
        match op_type {
            0 => return format!("LSL R{},R{},#{}", rd, rs, offset),
            1 => return format!("LSR R{},R{},#{}", rd, rs, if offset==0 {32} else {offset}),
            2 => return format!("ASR R{},R{},#{}", rd, rs, if offset==0 {32} else {offset}),
            _ => {}
        }
    }
    
    // Format 2: Add/subtract (00011 ooo xxx xxx xxx)
    if op >> 10 == 0b00011 {
        let is_imm = op & (1<<10) == 0;
        let is_sub = op & (1<<9) != 0;
        let rn_off = (op >> 6) & 7;
        let rs = (op >> 3) & 7;
        let rd = op & 7;
        let src = if is_imm { format!("#{}", rn_off) } else { format!("R{}", rn_off) };
        let op_name = if is_sub {"SUB"} else {"ADD"};
        return format!("{} R{},R{},{}", op_name, rd, rs, src);
    }

    // Format 3: Move/compare/add/subtract immediate (001 oo xxx xxxxxxx)
    if op >> 13 == 0b001 {
        let op_type = (op >> 11) & 3;
        let rd = (op >> 8) & 7;
        let imm = op & 0xFF;
        match op_type {
            0 => return format!("MOV R{},#0x{:02X}", rd, imm),
            1 => return format!("CMP R{},#0x{:02X}", rd, imm),
            2 => return format!("ADD R{},#0x{:02X}", rd, imm),
            3 => return format!("SUB R{},#0x{:02X}", rd, imm),
            _ => {}
        }
    }

    // Format 4: ALU operations (010000 xxx xxx xxx xxx)
    if op >> 10 == 0b010000 {
        let op_type = (op >> 6) & 0xF;
        let rs = (op >> 3) & 7;
        let rd = op & 7;
        let name = match op_type {
            0=>"AND",1=>"EOR",2=>"LSL",3=>"LSR",4=>"ASR",5=>"ADC",6=>"SBC",
            7=>"ROR",8=>"TST",9=>"NEG",10=>"CMP",11=>"CMN",12=>"ORR",
            13=>"MUL",14=>"BIC",15=>"MVN",_=>"?"
        };
        return format!("{} R{},R{}", name, rd, rs);
    }

    // Format 5: Hi register operations / branch exchange (010001 xx ...)
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

    // Format 6: PC-relative load (01001 xxx xxxxxxxx)
    if op >> 11 == 0b01001 {
        let rd = (op >> 8) & 7;
        let word8 = (op & 0xFF) as u32;
        let addr = ((pc + 4) & !2) + word8 * 4;
        return format!("LDR R{},[PC,#0x{:02X}] ;=0x{:08X}", rd, word8*4, addr);
    }

    // Format 7/8: Load/store register offset (0101 xxx ...)
    if op >> 12 == 0b0101 {
        let op_type = (op >> 10) & 3;
        let ro = (op >> 6) & 7;
        let rb = (op >> 3) & 7;
        let rd = op & 7;
        let byte = op & (1<<10) != 0;
        let load = op & (1<<11) != 0;
        match (load, byte) {
            (false,false) if op_type==0 => return format!("STR R{},[R{},R{}]", rd, rb, ro),
            (true,false) if op_type==0 => return format!("STRH R{},[R{},R{}]", rd, rb, ro),
            _ => {
                let name = match (load,byte) {
                    (false,false)=>"STRB",(true,false)=>"LDR",(false,true)=>"STRB",(true,true)=>"LDRB",_=>"?"
                };
                return format!("{} R{},[R{},R{}]", name, rd, rb, ro);
            }
        }
    }

    // Format 9: Load/store with immediate offset (011x xxx ...)
    if op >> 13 == 0b011 {
        let load = op & (1<<11) != 0;
        let byte = op & (1<<12) == 0;
        let offset5 = ((op >> 6) & 0x1F) as u32;
        let rb = (op >> 3) & 7;
        let rd = op & 7;
        if byte {
            let name = if load {"LDRB"} else {"STRB"};
            return format!("{} R{},[R{},#0x{:02X}]", name, rd, rb, offset5);
        } else {
            let name = if load {"LDR"} else {"STR"};
            return format!("{} R{},[R{},#0x{:02X}]", name, rd, rb, offset5*4);
        }
    }

    // Format 10: Load/store halfword (1000 xxx ...)
    if op >> 12 == 0b1000 {
        let load = op & (1<<11) != 0;
        let offset5 = ((op >> 6) & 0x1F) as u32;
        let rb = (op >> 3) & 7;
        let rd = op & 7;
        let name = if load {"LDRH"} else {"STRH"};
        return format!("{} R{},[R{},#0x{:02X}]", name, rd, rb, offset5*2);
    }

    // Format 11: SP-relative load/store (1001x xxx xxxxxxxx)
    if op >> 12 == 0b1001 {
        let load = op & (1<<11) != 0;
        let rd = (op >> 8) & 7;
        let word8 = (op & 0xFF) as u32;
        let name = if load {"LDR"} else {"STR"};
        return format!("{} R{},[SP,#0x{:02X}]", name, rd, word8*4);
    }

    // Format 12: Load address (1010 x xxx xxxxxxxx)
    if op >> 11 == 0b10100 {
        let rd = (op >> 8) & 7;
        let word8 = (op & 0xFF) as u32;
        return format!("ADD R{},PC,#0x{:02X}", rd, word8*4);
    }
    if op >> 11 == 0b10101 {
        let rd = (op >> 8) & 7;
        let word8 = (op & 0xFF) as u32;
        return format!("ADD R{},SP,#0x{:02X}", rd, word8*4);
    }

    // Format 13: Adjust SP (10110000 xxxxxxxx)
    if op >> 8 == 0b10110000 {
        let s = op & (1<<7) != 0;
        let imm = (op & 0x7F) as u32 * 4;
        return format!("{} SP,#0x{:02X}", if s {"SUB"} else {"ADD"}, imm);
    }

    // Format 14: Push/pop (1011 x10x xxxxxxxxx)
    if op >> 9 == 0b1011 && ((op >> 8) & 1) == 0 {
        let l = op & (1<<11) != 0;
        let r = op & (1<<8) != 0;
        let mut regs = Vec::new();
        for i in 0..8 {
            if op & (1<<i) != 0 { regs.push(format!("R{}", i)); }
        }
        if r { regs.push("LR".to_string()); }
        let name = if l {"POP"} else {"PUSH"};
        let extra = if r { (if l {",PC"} else {",LR"}) } else { "" };
        return format!("{} {{{}}}", name, regs.join(","));
    }

    // Format 15: Multiple load/store (1100 xxxxxxxxxxxxx)
    if op >> 12 == 0b1100 {
        let load = op & (1<<11) != 0;
        let rb = (op >> 8) & 7;
        let mut regs = Vec::new();
        for i in 0..8 {
            if op & (1<<i) != 0 { regs.push(format!("R{}", i)); }
        }
        let name = if load {"LDMIA"} else {"STMIA"};
        return format!("{} R{}!,{{{}}}", name, rb, regs.join(","));
    }

    // Format 16: Conditional branch (1101 cccc xxxxxxxxx)
    if op >> 12 == 0b1101 {
        let cond = (op >> 8) & 0xF;
        if cond < 0xE {
            let offset = (op & 0xFF) as i8 as i32;
            let target = (pc as i32 + 4 + offset * 2) as u32;
            let cn = match cond {
                0=>"EQ",1=>"NE",2=>"CS",3=>"CC",4=>"MI",5=>"PL",
                6=>"VS",7=>"VC",8=>"HI",9=>"LS",10=>"GE",11=>"LT",
                12=>"GT",13=>"LE",_=>"?"
            };
            return format!("B{} 0x{:08X}", cn, target);
        }
        if cond == 0xE { return "UDEF".to_string(); }
        // SWI
        let num = op & 0xFF;
        return format!("SWI #0x{:02X}", num);
    }

    // Format 17: Software interrupt (1101 1111 xxxxxxxx)
    if op >> 8 == 0b11011111 {
        return format!("SWI #0x{:02X}", op & 0xFF);
    }

    // Format 18: Unconditional branch (11100 xxxxxxxxxxxxx)
    if op >> 11 == 0b11100 {
        let offset = (op & 0x7FF) as u32;
        let offset = if offset & 0x400 != 0 { (offset | 0xFFFFF800u32) as i32 } else { offset as i32 };
        let target = (pc as i32 + 4 + offset * 2) as u32;
        return format!("B 0x{:08X}", target);
    }

    // Format 19: Long branch with link (111x x xxxxxxxxxxxxx)
    if op >> 11 == 0b11110 {
        let offset = ((op & 0x7FF) as u32) << 12;
        let offset = if offset & 0x00400000 != 0 { (offset | 0xFF800000u32) as i32 } else { offset as i32 };
        return format!("BLHi #0x{:08X}", offset);
    }
    if op >> 11 == 0b11111 {
        let offset = op & 0x7FF;
        return format!("BLLo #0x{:03X}", offset);
    }

    format!(".hword 0x{:04X}", op)
}
