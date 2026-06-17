use rgba::Gba;

fn main() {
    let rom = std::fs::read("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    // Dump Thumb halfwords at key execution ranges
    let ranges = [
        (0x080D0930u32, 0x080D0970u32, "Hot decompression loop"),
        (0x080D0BE0u32, 0x080D0C10u32, "Tile-writing loop"),
        (0x080D08B0u32, 0x080D0920u32, "Decompression inner"),
        (0x080D0CF0u32, 0x080D0D60u32, "After gap - loaded code?"),
    ];

    for (start, end, name) in ranges {
        println!("\n=== {} ({:#010X}-{:#010X}) ===", name, start, end);
        let mut offset = (start - 0x08000000) as usize;
        while offset < (end - 0x08000000) as usize {
            let hw = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
            let pc = 0x08000000 + offset as u32;

            // Simple Thumb disassembly
            let disasm = decode_thumb(pc, hw);
            println!("{:#010X}: {:04X}  {}", pc, hw, disasm);

            offset += 2;
        }
    }
}

fn decode_thumb(pc: u32, hw: u16) -> String {
    let bits15_14 = (hw >> 14) & 3;
    let bits15_13 = (hw >> 13) & 7;
    let bits15_12 = (hw >> 12) & 0xF;
    let bits15_11 = (hw >> 11) & 0x1F;

    match bits15_11 {
        0b00000 => {
            // LSL Rd, Rm, #imm5
            let imm5 = (hw >> 6) & 0x1F;
            let rm = (hw >> 3) & 7;
            let rd = hw & 7;
            if imm5 == 0 {
                format!("MOV R{}, R{}", rd, rm)
            } else {
                format!("LSL R{}, R{}, #{}", rd, rm, imm5)
            }
        }
        0b00001 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rm = (hw >> 3) & 7;
            let rd = hw & 7;
            format!(
                "LSR R{}, R{}, #{}",
                rd,
                rm,
                if imm5 == 0 { 32 } else { imm5 }
            )
        }
        0b00010 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rm = (hw >> 3) & 7;
            let rd = hw & 7;
            format!(
                "ASR R{}, R{}, #{}",
                rd,
                rm,
                if imm5 == 0 { 32 } else { imm5 }
            )
        }
        0b00011 => {
            let op = (hw >> 9) & 3;
            let rm = (hw >> 6) & 7;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            match op {
                0 => format!("ADD R{}, R{}, R{}", rd, rn, rm),
                1 => format!("SUB R{}, R{}, R{}", rd, rn, rm),
                2 => format!("ADD R{}, R{}, R{} (with carry)", rd, rn, rm),
                3 => format!("SUB R{}, R{}, R{} (with borrow)", rd, rn, rm),
                _ => format!("UNKNOWN ADD/SUB"),
            }
        }
        0b00100 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("MOV R{}, #{}", rd, imm8)
        }
        0b00101 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("CMP R{}, #{}", rd, imm8)
        }
        0b00110 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("ADD R{}, #{}", rd, imm8)
        }
        0b00111 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("SUB R{}, #{}", rd, imm8)
        }
        _ if bits15_12 == 0b0100 => {
            let bits11_10 = (hw >> 10) & 3;
            match bits11_10 {
                0 => {
                    // ALU operations
                    let op = (hw >> 6) & 0xF;
                    let rm = (hw >> 3) & 7;
                    let rd = hw & 7;
                    let op_name = match op {
                        0 => "AND",
                        1 => "EOR",
                        2 => "LSL",
                        3 => "LSR",
                        4 => "ASR",
                        5 => "ADC",
                        6 => "SBC",
                        7 => "ROR",
                        8 => "TST",
                        9 => "NEG",
                        10 => "CMP",
                        11 => "CMN",
                        12 => "ORR",
                        13 => "MUL",
                        14 => "BIC",
                        15 => "MVN",
                        _ => "???",
                    };
                    format!("{} R{}, R{}", op_name, rd, rm)
                }
                1 => {
                    // Hi register operations / branch exchange
                    let op = (hw >> 8) & 3;
                    let h1 = (hw >> 7) & 1;
                    let h2 = (hw >> 6) & 1;
                    let rm = ((h2 as u32) << 3) | ((hw >> 3) & 7) as u32;
                    let rd = ((h1 as u32) << 3) | (hw & 7) as u32;
                    match op {
                        0 => format!("ADD R{}, R{}", rd, rm),
                        1 => format!("CMP R{}, R{}", rd, rm),
                        2 => format!("MOV R{}, R{}", rd, rm),
                        3 => {
                            if rm >= 8 {
                                format!("BX R{}", rm)
                            } else {
                                format!("BLX R{}", rm)
                            }
                        }
                        _ => "???".to_string(),
                    }
                }
                2 => {
                    // Load/store with PC-relative offset
                    let rd = (hw >> 8) & 7;
                    let word8 = (hw & 0xFF) as u32;
                    let addr = (pc & !3) + 4 + word8 * 4;
                    format!("LDR R{}, [PC+#{:#X}] ({:#010X})", rd, word8 * 4, addr)
                }
                3 => {
                    // Load/store with SP-relative offset
                    let l = (hw >> 11) & 1;
                    let rd = (hw >> 8) & 7;
                    let word8 = (hw & 0xFF) as u32;
                    let op = if l != 0 { "LDR" } else { "STR" };
                    format!("{} R{}, [SP+#{:#X}]", op, rd, word8 * 4)
                }
                _ => format!("UNKNOWN 0100"),
            }
        }
        0b0101 => {
            let l = (hw >> 11) & 1;
            let b = (hw >> 10) & 1;
            let rm = (hw >> 6) & 7;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            let op = match (l, b) {
                (0, 0) => "STR",
                (0, 1) => "STRB",
                (1, 0) => "LDR",
                (1, 1) => "LDRB",
                _ => "???",
            };
            format!("{} R{}, [R{}, R{}]", op, rd, rn, rm)
        }
        0b01100 => {
            let l = (hw >> 11) & 1;
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            let op = if l != 0 { "LDR" } else { "STR" };
            format!("{} R{}, [R{}, #{}]", op, rd, rn, imm5 * 4)
        }
        0b01101 => {
            let l = (hw >> 11) & 1;
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            let op = if l != 0 { "LDRB" } else { "STRB" };
            format!("{} R{}, [R{}, #{}]", op, rd, rn, imm5)
        }
        0b01110 => {
            let l = (hw >> 11) & 1;
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            let op = if l != 0 { "LDRH" } else { "STRH" };
            format!("{} R{}, [R{}, #{}]", op, rd, rn, imm5 * 2)
        }
        0b01111 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            if (hw >> 11) & 1 != 0 {
                format!("LDRSB R{}, [R{}, #{}]", rd, rn, imm5)
            } else {
                format!("STR R{}, [R{}, #{}]", rd, rn, imm5 * 4)
            }
        }
        0b10000 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            if (hw >> 11) & 1 != 0 {
                format!("LDRSH R{}, [R{}, #{}]", rd, rn, imm5)
            } else {
                format!("STRH R{}, [R{}, #{}]", rd, rn, imm5 * 2)
            }
        }
        0b10001 => {
            let l = (hw >> 11) & 1;
            let rb = (hw >> 8) & 7;
            let rlist = hw & 0xFF;
            let op = if l != 0 { "LDMIA" } else { "STMIA" };
            let mut regs = Vec::new();
            for i in 0..8 {
                if rlist & (1 << i) != 0 {
                    regs.push(format!("R{}", i));
                }
            }
            format!("{} R{}!, {{{}}}", op, rb, regs.join(", "))
        }
        0b10100 => {
            let l = (hw >> 11) & 1;
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            let op = if l != 0 { "LDR" } else { "STR" };
            format!("{} R{}, [SP, #{}]", op, rd, imm8 * 4)
        }
        0b10110 if (hw >> 10) & 3 == 0 => {
            let imm8 = hw & 0xFF;
            format!("ADD SP, #{}", (imm8 as i8 as i16) * 4)
        }
        0b10110 if (hw >> 10) & 3 == 2 => {
            format!("PUSH {{{}}}", if hw & 0x100 != 0 { "LR, " } else { "" })
        }
        0b10111 if (hw >> 10) & 3 == 0 => {
            format!("POP {{{}}}", if hw & 0x100 != 0 { "PC, " } else { "" })
        }
        0b11000 => {
            let cond = (hw >> 8) & 0xF;
            let offset = hw & 0xFF;
            let cond_names = [
                "EQ", "NE", "CS", "CC", "MI", "PL", "VS", "VC", "HI", "LS", "GE", "LT", "GT", "LE",
                "", "NV",
            ];
            let signed_off = (offset as i8 as i16) * 2 + 4;
            let target = (pc as i32 + signed_off as i32) as u32;
            if cond < 14 {
                format!("B{} {:#010X}", cond_names[cond as usize], target)
            } else {
                format!("Bcond?? {:#010X}", target)
            }
        }
        0b11010 => {
            let offset = (hw & 0x7FF) as i32;
            let signed_off = ((offset << 21) >> 20) + 4;
            let target = (pc as i32 + signed_off) as u32;
            format!("B {:#010X}", target)
        }
        0b11011 => {
            let offset = (hw & 0x7FF) as u32;
            format!("BL prefix {:#010X}", offset)
        }
        0b11100 => {
            let offset = (hw & 0x7FF) as u32;
            format!("BL suffix {:#010X}", offset)
        }
        0b11101 => {
            format!("SWI {}", hw & 0xFF)
        }
        _ => {
            format!("UNKNOWN ({:#06X})", hw)
        }
    }
}
