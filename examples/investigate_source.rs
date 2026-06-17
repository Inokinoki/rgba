use rgba::Gba;

const ROM_PATH: &str = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";

fn main() {
    let rom = std::fs::read(ROM_PATH).unwrap();

    println!("=== Phase 1: Static Disassembly around 0x08001B0C ===\n");

    let center = 0x1B0Cusize;
    let start = center.saturating_sub(160);
    let end = (center + 160).min(rom.len() - 2);

    for off in (start..end).step_by(2) {
        let addr = 0x08000000 + off;
        let hw = u16::from_le_bytes([rom[off], rom[off + 1]]);
        let disasm = decode_thumb(addr as u32, hw);
        let marker = if addr == 0x08001B0C {
            " <<<< STRH TO VRAM"
        } else {
            ""
        };
        println!("  {:08X}: {:04X}  {}{}", addr, hw, disasm, marker);
    }

    println!("\n=== Literal pool at 0x08001E30 ===");
    for off in (0x1E30..0x1E50).step_by(4) {
        let val = u32::from_le_bytes([rom[off], rom[off + 1], rom[off + 2], rom[off + 3]]);
        println!("  {:08X}: {:08X}", 0x08000000 + off as u32, val);
    }

    println!("\n=== Phase 2: Capture ALL writes at 0x08001B0C ===\n");

    let mut gba = Gba::new();
    gba.load_rom_path(ROM_PATH).unwrap();
    let mut fb = vec![0u32; 240 * 160];

    let target_pc: u32 = 0x08001B0C;
    let mut step_count: u64 = 0;
    let max_steps = 100_000_000u64;

    let mut all_writes: Vec<(u32, u32, u32, u32, u32, u32)> = Vec::new();
    let mut first_sp: Option<u32> = None;
    let mut first_ip: Option<u32> = None;

    for frame in 0..240u32 {
        for _step in 0..500_000 {
            let pc = gba.cpu.get_instruction_pc();
            if pc == target_pc {
                let r0 = gba.cpu.get_reg(0);
                let r1 = gba.cpu.get_reg(1);
                let r2 = gba.cpu.get_reg(2);
                let r3 = gba.cpu.get_reg(3);
                let sp = gba.cpu.get_reg(13);
                let ip = gba.cpu.get_reg(12);

                if first_sp.is_none() {
                    first_sp = Some(sp);
                }
                if first_ip.is_none() {
                    first_ip = Some(ip);
                }

                if all_writes.len() < 1024 {
                    all_writes.push((r0, r1, r2, r3, sp, ip));
                }

                if all_writes.len() == 1024 {
                    println!(
                        "  Captured all 1024 block-0 writes at frame {} step {}!",
                        frame, step_count
                    );
                }
            }
            gba.step();
            step_count += 1;
        }

        if all_writes.len() >= 1024 {
            println!("  Running remaining {} frames fast...", 240 - frame - 1);
            for _ in (frame + 1)..240 {
                gba.run_frame_parallel(&mut fb);
            }
            break;
        }

        if step_count >= max_steps {
            println!(
                "  {} steps exhausted at frame {}, got {} writes",
                max_steps,
                frame,
                all_writes.len()
            );
            break;
        }

        if frame % 20 == 0 {
            println!(
                "  frame {}... ({}M steps, {} writes)",
                frame,
                step_count / 1_000_000,
                all_writes.len()
            );
        }
    }

    gba.sync_ppu_full();

    if all_writes.is_empty() {
        println!("  No writes captured!");
        return;
    }

    let sp = first_sp.unwrap();
    let ip = first_ip.unwrap();
    println!("\n  SP={:08X} IP(VRAM base)={:04X}", sp, ip);

    println!("\n=== Phase 3: Analyze writes ===\n");

    let (first_w, ..) = all_writes[0];
    println!(
        "  First write value: {:04X} (tile {} pal {})",
        first_w,
        first_w & 0x3FF,
        (first_w >> 12) & 0xF
    );

    let mut pal0_count = 0u32;
    let mut pal11_count = 0u32;
    let mut other_pal_count = 0u32;
    for (val, _, _, _, _, _) in &all_writes {
        let pal = (*val >> 12) & 0xF;
        match pal {
            0 => pal0_count += 1,
            11 => pal11_count += 1,
            _ => other_pal_count += 1,
        }
    }
    println!(
        "  Palette distribution: pal0={} pal11={} other={}",
        pal0_count, pal11_count, other_pal_count
    );

    println!("\n  First 64 writes (outer 0 and 1, block 0):");
    for (i, (val, dest, r2, src, _, _)) in all_writes.iter().enumerate().take(64) {
        let tile = val & 0x3FF;
        let pal = (val >> 12) & 0xF;
        let outer = i / 32;
        let inner = i % 32;
        if i % 16 == 0 {
            print!("\n    o={} i={:2}-: ", outer, inner);
        }
        print!("{:04X}(t{}p{}) ", val, tile, pal);
    }
    println!();

    println!("\n  All 32 writes for outer=0, block 0:");
    for (i, (val, _, _, src, _, _)) in all_writes.iter().enumerate().take(32) {
        let tile = val & 0x3FF;
        let pal = (val >> 12) & 0xF;
        println!(
            "    inner={:2}: val={:04X} tile={} pal={} src={:08X}",
            i, val, tile, pal, src
        );
    }

    let vram_copy: Vec<u8> = gba.mem.vram().to_vec();
    println!("\n=== Phase 4: Compare writes with final VRAM ===\n");

    let mut vram_match = 0u32;
    let mut vram_mismatch = 0u32;
    for (i, (val, _, r2, _, _, _)) in all_writes.iter().enumerate() {
        let vram_off = (*r2 as usize);
        if vram_off + 1 < vram_copy.len() {
            let vram_val = u16::from_le_bytes([vram_copy[vram_off], vram_copy[vram_off + 1]]);
            if *val == vram_val as u32 {
                vram_match += 1;
            } else {
                if vram_mismatch < 10 {
                    println!(
                        "  Mismatch write[{}]: wrote {:04X}, VRAM has {:04X} at +{:04X}",
                        i, val, vram_val, r2
                    );
                }
                vram_mismatch += 1;
            }
        }
    }
    println!(
        "  VRAM comparison: {} match, {} mismatch",
        vram_match, vram_mismatch
    );

    println!("\n=== Phase 5: Source data at time of write ===\n");

    let iwram: Vec<u8> = gba.mem.iwram().to_vec();
    let ewram: Vec<u8> = gba.mem.wram().to_vec();

    let src = all_writes[0].3;
    println!("  First source address was: {:08X}", src);
    println!("  Current IWRAM at that address (likely overwritten by now):");
    for i in 0..64u32 {
        let a = src.wrapping_add(i * 2);
        let v = read_mem_half(&iwram, &ewram, a);
        if i % 16 == 0 {
            print!("\n    ");
        }
        print!("{:04X}(t{}p{}) ", v, v & 0x3FF, (v >> 12) & 0xF);
    }
    println!();

    if pal0_count > 0 {
        println!("\n=== VERDICT ===");
        println!(
            "  {} out of {} writes have palette=0",
            pal0_count,
            all_writes.len()
        );
        println!("  Interior screen entries have palette=0 AT THE TIME OF WRITING.");
        println!("  The copy loop faithfully copies whatever is in the source buffer.");
        println!("  The bug is UPSTREAM: the source data in IWRAM was already wrong");
        println!("  before the copy loop at 0x08001B0C executed.");
        println!("\n  Source data comes from IWRAM at SP-based addresses.");
        println!("  The function that FILLS this source buffer (before the copy loop)");
        println!("  is the one producing palette=0 instead of palette=11.");
        println!(
            "  LR={:08X} points to the caller of this function.",
            all_writes[0].5
        );
    } else {
        println!("\n  All writes had non-zero palette. Bug may be elsewhere.");
    }
}

fn read_mem_half(iwram: &[u8], ewram: &[u8], addr: u32) -> u16 {
    if addr >= 0x03000000 && addr < 0x03008000 {
        let off = (addr - 0x03000000) as usize;
        if off + 1 < iwram.len() {
            return u16::from_le_bytes([iwram[off], iwram[off + 1]]);
        }
    } else if addr >= 0x02000000 && addr < 0x02040000 {
        let off = (addr - 0x02000000) as usize;
        if off + 1 < ewram.len() {
            return u16::from_le_bytes([ewram[off], ewram[off + 1]]);
        }
    }
    0xFFFF
}

fn decode_thumb(pc: u32, hw: u16) -> String {
    match (hw >> 11) & 0x1F {
        0b00000 => {
            let i = (hw >> 6) & 0x1F;
            if i == 0 {
                format!("MOV R{},R{}", hw & 7, (hw >> 3) & 7)
            } else {
                format!("LSL R{},R{},#{}", hw & 7, (hw >> 3) & 7, i)
            }
        }
        0b00001 => format!(
            "LSR R{},R{},#{}",
            hw & 7,
            (hw >> 3) & 7,
            if (hw >> 6) & 0x1F == 0 {
                32
            } else {
                (hw >> 6) & 0x1F
            }
        ),
        0b00010 => format!(
            "ASR R{},R{},#{}",
            hw & 7,
            (hw >> 3) & 7,
            if (hw >> 6) & 0x1F == 0 {
                32
            } else {
                (hw >> 6) & 0x1F
            }
        ),
        0b00011 => {
            let op = (hw >> 9) & 3;
            let rd = hw & 7;
            let rn = (hw >> 3) & 7;
            let rm = (hw >> 6) & 7;
            match op {
                0 => format!("ADD R{},R{},R{}", rd, rn, rm),
                1 => format!("SUB R{},R{},R{}", rd, rn, rm),
                _ => format!("OP{}", op),
            }
        }
        0b00100 => format!("MOV R{},#{}", (hw >> 8) & 7, hw & 0xFF),
        0b00101 => format!("CMP R{},#{}", (hw >> 8) & 7, hw & 0xFF),
        0b00110 => format!("ADD R{},#{}", (hw >> 8) & 7, hw & 0xFF),
        0b00111 => format!("SUB R{},#{}", (hw >> 8) & 7, hw & 0xFF),
        0b01100 => format!(
            "{} R{},[R{},#{}]",
            if (hw >> 11) & 1 != 0 { "LDR" } else { "STR" },
            hw & 7,
            (hw >> 3) & 7,
            ((hw >> 6) & 0x1F) * 4
        ),
        0b01110 => format!(
            "{} R{},[R{},#{}]",
            if (hw >> 11) & 1 != 0 { "LDRH" } else { "STRH" },
            hw & 7,
            (hw >> 3) & 7,
            ((hw >> 6) & 0x1F) * 2
        ),
        0b01101 => format!(
            "{}{} R{},[R{},#{}]",
            if (hw >> 11) & 1 != 0 { "LDR" } else { "STR" },
            if (hw >> 10) & 1 != 0 { "B" } else { "" },
            hw & 7,
            (hw >> 3) & 7,
            (hw >> 6) & 0x1F
        ),
        _ if (hw >> 12) & 0xF == 4 => match (hw >> 10) & 3 {
            0 => {
                let ops = [
                    "AND", "EOR", "LSL", "LSR", "ASR", "ADC", "SBC", "ROR", "TST", "NEG", "CMP",
                    "CMN", "ORR", "MUL", "BIC", "MVN",
                ];
                format!(
                    "{} R{},R{}",
                    ops[((hw >> 6) & 0xF) as usize],
                    hw & 7,
                    (hw >> 3) & 7
                )
            }
            1 => {
                let op = (hw >> 8) & 3;
                let h1 = (hw >> 7) & 1;
                let h2 = (hw >> 6) & 1;
                let rm = (h2 as u32) << 3 | ((hw >> 3) & 7) as u32;
                let rd = (h1 as u32) << 3 | (hw & 7) as u32;
                match op {
                    0 => format!("ADD R{},R{}", rd, rm),
                    2 => format!("MOV R{},R{}", rd, rm),
                    3 => format!("BX R{}", rm),
                    _ => format!("Hi{}", op),
                }
            }
            2 => {
                let rd = (hw >> 8) & 7;
                let w = (hw & 0xFF) as u32;
                format!("LDR R{},[{:#010X}]", rd, (pc & !3) + 4 + w * 4)
            }
            3 => format!(
                "{} R{},[SP,#{}]",
                if (hw >> 11) & 1 != 0 { "LDR" } else { "STR" },
                (hw >> 8) & 7,
                (hw & 0xFF) as u32 * 4
            ),
            _ => "?".into(),
        },
        0b0101 => format!(
            "{}{} R{},[R{},R{}]",
            if (hw >> 11) & 1 != 0 { "LDR" } else { "STR" },
            if (hw >> 10) & 1 != 0 { "B" } else { "" },
            hw & 7,
            (hw >> 3) & 7,
            (hw >> 6) & 7
        ),
        0b10001 => {
            let mut r = Vec::new();
            for i in 0..8 {
                if hw & (1 << i) != 0 {
                    r.push(format!("R{}", i));
                }
            }
            format!(
                "{} R{}!,{{{}}}",
                if (hw >> 11) & 1 != 0 {
                    "LDMIA"
                } else {
                    "STMIA"
                },
                (hw >> 8) & 7,
                r.join(",")
            )
        }
        0b10100 => format!(
            "{} R{},[SP,#{}]",
            if (hw >> 11) & 1 != 0 { "LDR" } else { "STR" },
            (hw >> 8) & 7,
            (hw & 0xFF) as u32 * 4
        ),
        0b10110 if (hw >> 10) & 3 == 2 => {
            let mut r = Vec::new();
            for i in 0..8 {
                if hw & (1 << i) != 0 {
                    r.push(format!("R{}", i));
                }
            }
            format!(
                "PUSH {{{}{}}}",
                if hw & 0x100 != 0 { "LR," } else { "" },
                r.join(",")
            )
        }
        0b10111 if (hw >> 10) & 3 == 0 => {
            let mut r = Vec::new();
            for i in 0..8 {
                if hw & (1 << i) != 0 {
                    r.push(format!("R{}", i));
                }
            }
            format!(
                "POP {{{}{}}}",
                if hw & 0x100 != 0 { "PC," } else { "" },
                r.join(",")
            )
        }
        0b11000 => {
            let c = (hw >> 8) & 0xF;
            let cn = [
                "EQ", "NE", "CS", "CC", "MI", "PL", "VS", "VC", "HI", "LS", "GE", "LT", "GT", "LE",
                "",
            ];
            let t = (pc as i32 + ((hw & 0xFF) as i8 as i16 * 2 + 4) as i32) as u32;
            format!("B{} {:#010X}", cn[c as usize], t)
        }
        0b11010 => {
            let o = (hw & 0x7FF) as i32;
            let s = ((o << 21) >> 20) + 4;
            format!("B {:#010X}", (pc as i32 + s) as u32)
        }
        0b11011 => format!("BL-hi {:X}", (hw & 0x7FF) as u32),
        0b11100 => format!("BL-lo {:X}", (hw & 0x7FF) as u32),
        0b11101 => format!("SWI {}", hw & 0xFF),
        0b10000 => {
            let i = (hw >> 6) & 0x1F;
            if (hw >> 11) & 1 != 0 {
                format!("LDRSH R{},[R{},#{}]", hw & 7, (hw >> 3) & 7, i)
            } else {
                format!("STRH R{},[R{},#{}]", hw & 7, (hw >> 3) & 7, i * 2)
            }
        }
        0b10110 if (hw >> 10) & 3 == 0 => format!("ADD SP,#{}", ((hw & 0xFF) as i8 as i16) * 4),
        _ => format!("?? ({:#06X})", hw),
    }
}
