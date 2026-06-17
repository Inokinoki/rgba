use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem.rom();

    // Decode THUMB instructions around 0x0805E400-0x0805E500
    let base = 0x0805E400;
    let off = (base - 0x08000000) as usize;

    println!("=== THUMB code at {:08X} ===", base);
    for i in 0..80 {
        let addr = base + i * 2;
        let hw = u16::from_le_bytes([rom[off + i * 2], rom[off + i * 2 + 1]]);
        let decoded = decode_thumb(hw);
        println!("{:08X}: {:04X}  {}", addr, hw, decoded);
    }
}

fn decode_thumb(hw: u16) -> String {
    let op = (hw >> 8) as u8;
    match hw >> 11 {
        0b00000 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rm = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("LSL r{}, r{}, #{}", rd, rm, imm5)
        }
        0b00001 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rm = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("LSR r{}, r{}, #{}", rd, rm, imm5)
        }
        0b00010 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rm = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("ASR r{}, r{}, #{}", rd, rm, imm5)
        }
        0b00011 => {
            let sub_op = (hw >> 9) & 3;
            let rm = (hw >> 6) & 7;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            match sub_op {
                0 => format!("ADD r{}, r{}, r{}", rd, rn, rm),
                1 => format!("SUB r{}, r{}, r{}", rd, rn, rm),
                2 => format!("ADD r{}, r{}, #imm", rd, rn),
                3 => format!("SUB r{}, r{}, #imm", rd, rn),
                _ => format!("???"),
            }
        }
        0b00100 | 0b00101 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("MOV r{}, #0x{:02X}", rd, imm8)
        }
        0b00110 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("CMP r{}, #0x{:02X}", rd, imm8)
        }
        0b00111 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("ADD r{}, #0x{:02X}", rd, imm8)
        }
        0b01000 => {
            let op = (hw >> 10) & 0xF;
            match op {
                0b0000 => format!("AND r{}, r{}", (hw >> 3) & 7, (hw >> 3) & 7),
                0b0001 => format!("EOR r{}, r{}", (hw >> 3) & 7, (hw >> 3) & 7),
                0b0010 => format!("MOV r{}, r{} (high)", (hw >> 3) & 7, hw & 7),
                0b0011 => format!("BX r{}", (hw >> 3) & 7),
                0xC => format!("UMULL"),
                _ => format!("ALU op {} r{}r{}", op, (hw >> 3) & 7, hw & 7),
            }
        }
        0b01001 => {
            let rd = (hw >> 8) & 7;
            let imm8 = hw & 0xFF;
            format!("LDR r{}, [PC, #0x{:02X}]", rd, imm8 * 4)
        }
        0b01010 => {
            let rd = (hw >> 8) & 7;
            let rn = (hw >> 6) & 7;
            let rm = (hw >> 3) & 7;
            format!("STR r{}, [r{}, r{}]", rd, rn, rm)
        }
        0b01011 => {
            let rd = (hw >> 8) & 7;
            let rn = (hw >> 6) & 7;
            let rm = (hw >> 3) & 7;
            format!("LDRB r{}, [r{}, r{}]", rd, rn, rm)
        }
        0b01100 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("STRB r{}, [r{}, #0x{:02X}]", rd, rn, imm5)
        }
        0b01110 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("LDRB r{}, [r{}, #0x{:02X}]", rd, rn, imm5)
        }
        0b10000 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("STRH r{}, [r{}, #0x{:02X}]", rd, rn, imm5 * 2)
        }
        0b10001 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("LDRH r{}, [r{}, #0x{:02X}]", rd, rn, imm5 * 2)
        }
        0b10010 => {
            let imm5 = (hw >> 6) & 0x1F;
            let rn = (hw >> 3) & 7;
            let rd = hw & 7;
            format!("LDR r{}, [r{}, #0x{:02X}]", rd, rn, imm5 * 4)
        }
        0b10100 => {
            let imm8 = hw & 0xFF;
            let rd = (hw >> 8) & 7;
            format!("STR r{}, [SP, #0x{:02X}]", rd, imm8 * 4)
        }
        0b10101 => {
            let imm8 = hw & 0xFF;
            let rd = (hw >> 8) & 7;
            format!("LDR r{}, [SP, #0x{:02X}]", rd, imm8 * 4)
        }
        0b10110 => {
            let sub_op = (hw >> 9) & 3;
            match sub_op {
                0 => format!("ADD r{}, SP, #0x{:02X}", (hw >> 8) & 7, (hw & 0xFF) * 4),
                2 => format!("ADD SP, #0x{:02X}", (hw & 0x7F) * 4),
                3 => format!("ADD SP, #-0x{:02X}", (hw & 0x7F) * 4),
                _ => format!("???"),
            }
        }
        0b11000 => {
            let cond = (hw >> 8) & 0xF;
            let offset = (hw & 0xFF) as i8 as i16;
            let target = offset * 2;
            let cond_names = [
                "EQ", "NE", "CS", "CC", "MI", "PL", "VS", "VC", "HI", "LS", "GE", "LT", "GT", "LE",
                "", "",
            ];
            format!("B{} {:+}", cond_names[cond as usize], target)
        }
        0b11100 => {
            let offset = ((hw & 0x7FF) as i16) << 5 >> 5;
            let target = offset.wrapping_mul(2);
            format!("B {:+}", target)
        }
        0b11110 => {
            let imm11 = hw & 0x7FF;
            format!("BL prefix 0x{:03X}", imm11)
        }
        0b11111 => {
            let imm11 = hw & 0x7FF;
            format!("BL suffix 0x{:03X}", imm11)
        }
        0b10111 => {
            let l = (hw >> 11) & 1;
            let imm8 = hw & 0xFF;
            let rd = (hw >> 8) & 7;
            if l == 0 {
                format!("PUSH {{..r{}}}", rd)
            } else {
                format!("POP {{..r{}}}", rd)
            }
        }
        0b11010 => {
            format!("B {:+} (unconditional)", ((hw & 0x7FF) as i16) << 5 >> 5)
        }
        0b11011 => {
            let imm8 = hw & 0xFF;
            format!("SWI {}", imm8)
        }
        _ => {
            format!("??? (op={:05b})", hw >> 11)
        }
    }
}
