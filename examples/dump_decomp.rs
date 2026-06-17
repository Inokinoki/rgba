use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom();

    // Dump THUMB code at 0x080D0AE0 - 0x080D0C10
    // In ROM, 0x080D0AE0 maps to offset 0x000D0AE0 (strip 0x08000000)
    let base = 0x000D0AE0;
    println!("=== Decompression code at 0x080D0AE0 (THUMB) ===");
    for i in (0..0x120).step_by(2) {
        let offset = base + i;
        if offset + 2 > rom.len() {
            break;
        }
        let instr = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
        let pc: u32 = 0x080D0AE0 + i as u32;
        println!("{:08X}: {:04X}  {}", pc, instr, decode_thumb(instr, pc));
    }

    // Also dump the helper functions
    println!("\n=== Helper at 0x080D0920 (THUMB) ===");
    let base2 = 0x000D0920;
    for i in (0..0x40).step_by(2) {
        let offset = base2 + i;
        if offset + 2 > rom.len() {
            break;
        }
        let instr = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
        let pc: u32 = 0x080D0920 + i as u32;
        println!("{:08X}: {:04X}  {}", pc, instr, decode_thumb(instr, pc));
    }

    // Dump the call site area around 0x080D0A40-0x080D0AE0
    println!("\n=== Call site area 0x080D0A40-0x080D0AE0 ===");
    let base3 = 0x000D0A40;
    for i in (0..0xA0).step_by(2) {
        let offset = base3 + i;
        if offset + 2 > rom.len() {
            break;
        }
        let instr = u16::from_le_bytes([rom[offset], rom[offset + 1]]);
        let pc: u32 = 0x080D0A40 + i as u32;
        println!("{:08X}: {:04X}  {}", pc, instr, decode_thumb(instr, pc));
    }
}

fn decode_thumb(instr: u16, pc: u32) -> String {
    let cat = (instr >> 13) & 0x7;
    match cat {
        0 => {
            let op = (instr >> 11) & 0x3;
            match op {
                0 => {
                    let imm = (instr >> 6) & 0x1F;
                    let rd = instr & 7;
                    let rs = (instr >> 3) & 7;
                    format!("lsl r{}, r{}, #{}", rd, rs, imm)
                }
                1 => {
                    let imm = (instr >> 6) & 0x1F;
                    let rd = instr & 7;
                    let rs = (instr >> 3) & 7;
                    if imm == 0 {
                        format!("lsr r{}, r{}, #32", rd, rs)
                    } else {
                        format!("lsr r{}, r{}, #{}", rd, rs, imm)
                    }
                }
                2 => {
                    let imm = (instr >> 6) & 0x1F;
                    let rd = instr & 7;
                    let rs = (instr >> 3) & 7;
                    if imm == 0 {
                        format!("asr r{}, r{}, #32", rd, rs)
                    } else {
                        format!("asr r{}, r{}, #{}", rd, rs, imm)
                    }
                }
                3 => {
                    let op = (instr >> 9) & 0x3;
                    let rd = instr & 7;
                    let rs = (instr >> 3) & 7;
                    let rn = (instr >> 6) & 7;
                    match op {
                        0 => format!("add r{}, r{}, r{}", rd, rs, rn),
                        1 => format!("sub r{}, r{}, r{}", rd, rs, rn),
                        2 => format!("add r{}, r{}, #{}", rd, rs, rn * 4),
                        3 => format!("sub r{}, r{}, #{}", rd, rs, rn * 4),
                        _ => "?".into(),
                    }
                }
                _ => "?".into(),
            }
        }
        1 => {
            let op = (instr >> 11) & 0x3;
            let rd = (instr >> 8) & 7;
            let imm = instr & 0xFF;
            match op {
                0 => format!("mov r{}, #{}", rd, imm),
                1 => format!("cmp r{}, #{}", rd, imm as i8 as i16), // signed
                2 => format!("add r{}, #{}", rd, imm),
                3 => format!("sub r{}, #{}", rd, imm),
                _ => "?".into(),
            }
        }
        2 => {
            if (instr & 0xF800) == 0x4800 {
                let rd = (instr >> 8) & 7;
                let imm = (instr & 0xFF) * 4;
                format!("ldr r{}, [pc, #{}]", rd, imm)
            } else if (instr & 0xFC00) == 0x4400 {
                let op = (instr >> 8) & 0x3;
                let hd = (instr >> 7) & 1;
                let hs = (instr >> 6) & 1;
                let rd = (instr & 7) | (hd << 3);
                let rs = ((instr >> 3) & 7) | (hs << 3);
                match op {
                    0 => format!("add r{}, r{}", rd, rs),
                    1 => format!("cmp r{}, r{}", rd, rs),
                    2 => format!("mov r{}, r{}", rd, rs),
                    3 => format!("bx r{}", rs),
                    _ => "?".into(),
                }
            } else if (instr & 0xFC00) == 0x4000 {
                let op = (instr >> 6) & 0xF;
                let rd = instr & 7;
                let rs = (instr >> 3) & 7;
                match op {
                    0 => format!("and r{}, r{}", rd, rs),
                    1 => format!("eor r{}, r{}", rd, rs),
                    2 => format!("lsl r{}, r{}", rd, rs),
                    3 => format!("lsr r{}, r{}", rd, rs),
                    4 => format!("asr r{}, r{}", rd, rs),
                    5 => format!("adc r{}, r{}", rd, rs),
                    6 => format!("sbc r{}, r{}", rd, rs),
                    7 => format!("ror r{}, r{}", rd, rs),
                    8 => format!("tst r{}, r{}", rd, rs),
                    9 => format!("neg r{}, r{}", rd, rs),
                    10 => format!("cmp r{}, r{}", rd, rs),
                    11 => format!("cmn r{}, r{}", rd, rs),
                    12 => format!("orr r{}, r{}", rd, rs),
                    13 => format!("mul r{}, r{}", rd, rs),
                    14 => format!("bic r{}, r{}", rd, rs),
                    15 => format!("mvn r{}, r{}", rd, rs),
                    _ => "?".into(),
                }
            } else {
                let op = (instr >> 9) & 0x7;
                let ro = (instr >> 6) & 7;
                let rb = (instr >> 3) & 7;
                let rd = instr & 7;
                match op {
                    0 => format!("str r{}, [r{}, r{}]", rd, rb, ro),
                    1 => format!("strh r{}, [r{}, r{}]", rd, rb, ro),
                    2 => format!("strb r{}, [r{}, r{}]", rd, rb, ro),
                    3 => format!("ldrsb r{}, [r{}, r{}]", rd, rb, ro),
                    4 => format!("ldr r{}, [r{}, r{}]", rd, rb, ro),
                    5 => format!("ldrh r{}, [r{}, r{}]", rd, rb, ro),
                    6 => format!("ldrb r{}, [r{}, r{}]", rd, rb, ro),
                    7 => format!("ldrsh r{}, [r{}, r{}]", rd, rb, ro),
                    _ => "?".into(),
                }
            }
        }
        3 => {
            let load = (instr >> 11) & 1;
            let byte = (instr >> 12) & 1;
            let imm = ((instr >> 6) & 0x1F) * if byte != 0 { 1 } else { 4 };
            let rb = (instr >> 3) & 7;
            let rd = instr & 7;
            if load != 0 {
                if byte != 0 {
                    format!("ldrb r{}, [r{}, #{}]", rd, rb, imm)
                } else {
                    format!("ldr r{}, [r{}, #{}]", rd, rb, imm)
                }
            } else {
                if byte != 0 {
                    format!("strb r{}, [r{}, #{}]", rd, rb, imm)
                } else {
                    format!("str r{}, [r{}, #{}]", rd, rb, imm)
                }
            }
        }
        4 => {
            let load = (instr >> 11) & 1;
            let imm = ((instr >> 6) & 0x1F) * 2;
            let rb = (instr >> 3) & 7;
            let rd = instr & 7;
            if load != 0 {
                format!("ldrh r{}, [r{}, #{}]", rd, rb, imm)
            } else {
                format!("strh r{}, [r{}, #{}]", rd, rb, imm)
            }
        }
        5 => {
            let load = (instr >> 11) & 1;
            let rb = (instr >> 8) & 7;
            let rlist = instr & 0xFF;
            format!(
                "{} r{}, {{{:08b}}}",
                if load != 0 { "ldmia" } else { "stmia" },
                rb,
                rlist
            )
        }
        6 => {
            if (instr & 0xF000) == 0xC000 {
                let load = (instr >> 11) & 1;
                let rb = (instr >> 8) & 7;
                let rlist = instr & 0xFF;
                format!(
                    "{} r{}, {{{:08b}}}",
                    if load != 0 { "ldmia" } else { "stmia" },
                    rb,
                    rlist
                )
            } else if (instr & 0xF000) == 0xD000 {
                if (instr & 0xFF00) == 0xDF00 {
                    format!("swi #{}", instr & 0xFF)
                } else {
                    let cond = ((instr >> 8) & 0xF) as usize;
                    let offset = (instr & 0xFF) as i8 as i16 as i32 * 2;
                    let conds = [
                        "eq", "ne", "cs", "cc", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt",
                        "gt", "le", "al", "nv",
                    ];
                    let target = pc as i32 + 4 + offset;
                    format!("b{} {:08X}", conds[cond], target as u32)
                }
            } else {
                let offset = (instr & 0x7FF) as i32;
                let offset = if offset > 1023 { offset - 2048 } else { offset };
                let target = pc as i32 + 4 + offset * 2;
                format!("b {:08X}", target as u32)
            }
        }
        7 => {
            let top5 = (instr >> 11) & 0x1F;
            match top5 {
                0b11100 => {
                    let offset = (instr & 0x7FF) as i32;
                    let offset = if offset > 1023 { offset - 2048 } else { offset };
                    let target = pc as i32 + 4 + offset * 2;
                    format!("b {:08X}", target as u32)
                }
                0b11110 => {
                    let offset = (instr & 0x7FF) << 11;
                    format!("bl prefix {:05X}", offset)
                }
                0b11111 => {
                    let offset = instr & 0x7FF;
                    format!("bl suffix {:03X}", offset)
                }
                _ => format!("??? ({:04X})", instr),
            }
        }
        _ => "?".into(),
    }
}
