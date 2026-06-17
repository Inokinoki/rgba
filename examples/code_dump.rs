use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom();

    let base = 0x080D2F10 - 0x08000000;
    println!("=== ROM code at 0x080D2F10 (Thumb) ===");
    for i in 0..32 {
        let off = base as usize + i * 2;
        if off + 1 >= rom.len() {
            break;
        }
        let opcode = u16::from_le_bytes([rom[off], rom[off + 1]]);
        let addr = 0x080D2F10 + i * 2;
        println!("{:#010X}: {:04X}  {}", addr, opcode, decode_thumb(opcode));
    }

    let base2 = 0x08000576 - 0x08000000;
    println!("\n=== ROM code at 0x08000576 (caller, Thumb) ===");
    for i in 0..40 {
        let off = base2 as usize + i * 2;
        if off + 1 >= rom.len() {
            break;
        }
        let opcode = u16::from_le_bytes([rom[off], rom[off + 1]]);
        let addr = 0x08000576 + i * 2;
        println!("{:#010X}: {:04X}  {}", addr, opcode, decode_thumb(opcode));
    }

    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..260 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let intr = &gba.mem().interrupt;
    println!("\n=== Interrupt State ===");
    println!("IME: {}", intr.ime);
    println!("IE: {:#06X}", intr.ie.bits());
    println!("IF: {:#06X}", intr.if_raw.bits());
    println!("In interrupt: {}", intr.in_interrupt);
    println!("Should wake from halt: {}", intr.should_wake_from_halt());
}

fn decode_thumb(opcode: u16) -> &'static str {
    let cat = (opcode >> 13) & 7;
    match cat {
        0 => {
            let op = (opcode >> 11) & 3;
            match op {
                0 => "LSL imm",
                1 => "LSR imm",
                2 => "ASR imm",
                3 => {
                    if opcode & 0x0800 != 0 {
                        "SUB/SUB"
                    } else {
                        "ADD/SUB"
                    }
                }
                _ => "?",
            }
        }
        1 => {
            let op = (opcode >> 11) & 3;
            match op {
                0 => "MOV imm8",
                1 => "CMP imm8",
                2 => "ADD imm8",
                3 => "SUB imm8",
                _ => "?",
            }
        }
        2 => {
            let op = (opcode >> 10) & 7;
            match op {
                0 => "ALU op",
                1 => {
                    if opcode & 0x200 != 0 {
                        "BX/BLX"
                    } else {
                        "ALU op"
                    }
                }
                2 => "LDR (lit)",
                3 => {
                    if opcode & 0x1000 != 0 {
                        "STR/LDR imm"
                    } else {
                        "STR/LDR imm"
                    }
                }
                _ => "mem",
            }
        }
        3 => {
            if opcode & 0x1000 != 0 {
                "LDR/STR byte/half"
            } else {
                "STR byte"
            }
        }
        4 => {
            if opcode & 0x0800 != 0 {
                "LDR/STR reg"
            } else {
                "STR/LDR imm sp"
            }
        }
        5 => {
            let op = (opcode >> 11) & 3;
            match op {
                0 => "ADD pc/sp imm",
                1 => {
                    if opcode & 0x0800 != 0 {
                        "POP"
                    } else {
                        "PUSH"
                    }
                }
                2 => {
                    if opcode & 0x0800 != 0 {
                        "POP pc"
                    } else {
                        "PUSH lr"
                    }
                }
                _ => "?",
            }
        }
        6 => {
            let op = (opcode >> 8) & 0xF;
            if op == 0xDE {
                "UDEF"
            } else if opcode & 0x0800 != 0 {
                "B cond"
            } else {
                "B cond"
            }
        }
        7 => {
            let hi = (opcode >> 8) & 0x1F;
            match hi {
                0x00..=0x0F => "SWI",
                _ => {
                    if (opcode >> 11) == 0x1D {
                        "BL prefix"
                    } else if (opcode >> 11) == 0x1F {
                        "BL suffix"
                    } else if (opcode >> 11) == 0x1E {
                        "BLX suffix"
                    } else {
                        "B/BL unconditional"
                    }
                }
            }
        }
        _ => "?",
    }
}
