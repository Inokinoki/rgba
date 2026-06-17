use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Expand the trace range to capture more of the decompression, including the Huffman decoder
    gba.cpu_mut().decomp_trace_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];
    for frame in 0..300 {
        gba.run_frame_parallel(&mut framebuffer);
        if frame % 100 == 0 {
            eprintln!("Frame {}", frame);
        }
    }

    let trace = &gba.cpu().decomp_trace;
    eprintln!("Total trace entries: {}", trace.len());

    // The decompression code structure:
    // R9 = Huffman byte decoder function pointer
    // The literal path: decode two bytes via R9, combine as (byte2<<8)|byte1, STRH to output
    //
    // Key addresses:
    // 0x080D0BEA: STRH R4,[R1,#0] - literal path write
    //
    // Before this STRH, the code calls R9 twice to decode bytes.
    // R4 = (decoded_byte2 << 8) | decoded_byte1
    //
    // Let me find the sequence: look for instructions that set R4 before the STRH

    // Let me find the first literal write and trace backwards to understand the Huffman decode

    // First, find the first few literal STRH writes at 0x080D0BEA
    let mut literal_indices: Vec<usize> = Vec::new();
    for (i, &(pc, _opcode, regs)) in trace.iter().enumerate() {
        let abs_pc = pc & !1;
        if abs_pc == 0x080D0BEA && regs[1] >= 0x03006DD8 && regs[1] < 0x03007000 {
            literal_indices.push(i);
            if literal_indices.len() >= 20 {
                break;
            }
        }
    }

    // For each literal write, trace back to find the Huffman decode sequence
    println!("=== Tracing Huffman decode before literal STRH ===\n");
    for (n, &idx) in literal_indices.iter().enumerate() {
        let (pc, opcode, regs) = trace[idx];
        let abs_pc = pc & !1;
        let r1 = regs[1];
        let r4_val = regs[4] as u16;
        let tile = r4_val & 0x3FF;
        let pal = (r4_val >> 12) & 0xF;

        println!(
            "Literal #{n} at trace[{}]: STRH R4={:04X} (tile={} pal={}) to [{:08X}]",
            idx, r4_val, tile, pal, r1
        );

        // Trace back up to 30 instructions to find the Huffman decode
        let start = if idx > 30 { idx - 30 } else { 0 };
        println!("  Trace back from here:");
        for j in start..=idx {
            let (jpc, jopcode, jregs) = trace[j];
            let jabs_pc = jpc & !1;
            let thumb_mode = jpc & 1 != 0;
            let disasm = if thumb_mode {
                thumb_disasm(jabs_pc, jopcode as u16, &jregs)
            } else {
                format!("{:08X?}", jopcode)
            };
            println!("    [{:7}] {:08X}: {:04X} R0={:08X} R1={:08X} R2={:08X} R3={:08X} R4={:08X} R5={:08X} R6={:08X} R7={:08X} R8={:08X} R9={:08X}",
                     j, jabs_pc, jopcode as u16, jregs[0], jregs[1], jregs[2], jregs[3], 
                     jregs[4], jregs[5], jregs[6], jregs[7], jregs[8], jregs[9]);
        }
        println!();
    }

    // Now let's check: what is R9 pointing to?
    // And what is the first instruction sequence for the first few entries?
    if let Some(&first_idx) = literal_indices.first() {
        let (_, _, regs) = trace[first_idx];
        println!(
            "\n=== First literal: R9={:08X} (Huffman decoder function) ===",
            regs[9]
        );
    }
}

fn thumb_disasm(pc: u32, opcode: u16, regs: &[u32; 16]) -> String {
    let category = (opcode >> 13) & 0x7;
    match category {
        0b000 => {
            let op = (opcode >> 11) & 0x3;
            match op {
                0 => format!(
                    "LSL R{}, R{}, #{}",
                    opcode & 7,
                    (opcode >> 3) & 7,
                    (opcode >> 6) & 0x1F
                ),
                1 => format!(
                    "LSR R{}, R{}, #{}",
                    opcode & 7,
                    (opcode >> 3) & 7,
                    (opcode >> 6) & 0x1F
                ),
                2 => format!(
                    "ASR R{}, R{}, #{}",
                    opcode & 7,
                    (opcode >> 3) & 7,
                    (opcode >> 6) & 0x1F
                ),
                3 => {
                    let sub = (opcode >> 9) & 1 != 0;
                    let imm = (opcode >> 6) & 7;
                    if sub {
                        format!("SUB R{}, R{}, #{}", opcode & 7, (opcode >> 3) & 7, imm)
                    } else {
                        format!("ADD R{}, R{}, #{}", opcode & 7, (opcode >> 3) & 7, imm)
                    }
                }
                _ => format!("???"),
            }
        }
        0b001 => {
            let op = (opcode >> 11) & 0x3;
            let rd = (opcode >> 8) & 7;
            let imm8 = opcode & 0xFF;
            match op {
                0 => format!("MOV R{}, #{}", rd, imm8),
                1 => format!("CMP R{}, #{}", rd, imm8),
                2 => format!("ADD R{}, #{}", rd, imm8),
                3 => format!("SUB R{}, #{}", rd, imm8),
                _ => "???".into(),
            }
        }
        0b010 => {
            if (opcode & 0xF800) == 0x4800 {
                let rd = (opcode >> 8) & 7;
                let imm8 = (opcode & 0xFF) as u32;
                let addr = ((pc + 4) & !2) + (imm8 << 1);
                format!("LDR R{}, [PC+#{}] ; ={:#010X}", rd, imm8 << 1, addr)
            } else if (opcode & 0xFC00) == 0x4000 {
                let op = (opcode >> 6) & 0xF;
                let rs = (opcode >> 3) & 7;
                let rd = opcode & 7;
                match op {
                    0 => format!("AND R{}, R{}", rd, rs),
                    1 => format!("EOR R{}, R{}", rd, rs),
                    2 => format!("LSL R{}, R{}", rd, rs),
                    3 => format!("LSR R{}, R{}", rd, rs),
                    4 => format!("ASR R{}, R{}", rd, rs),
                    5 => format!("ADC R{}, R{}", rd, rs),
                    6 => format!("SBC R{}, R{}", rd, rs),
                    7 => format!("ROR R{}, R{}", rd, rs),
                    8 => format!("TST R{}, R{}", rd, rs),
                    9 => format!("NEG R{}, R{}", rd, rs),
                    10 => format!("CMP R{}, R{}", rd, rs),
                    11 => format!("CMN R{}, R{}", rd, rs),
                    12 => format!("ORR R{}, R{}", rd, rs),
                    13 => format!("MUL R{}, R{}", rd, rs),
                    14 => format!("BIC R{}, R{}", rd, rs),
                    15 => format!("MVN R{}, R{}", rd, rs),
                    _ => "???".into(),
                }
            } else if (opcode & 0xFC00) == 0x4400 {
                // Hi register ops / branch exchange
                let op = (opcode >> 8) & 0x3;
                let msb = (opcode >> 7) & 1;
                let rs = ((opcode >> 3) & 7) | (((opcode >> 6) & 1) << 3);
                let rd = (opcode & 7) | (msb << 3);
                match op {
                    0 => format!("ADD R{}, R{}", rd, rs),
                    1 => format!("CMP R{}, R{}", rd, rs),
                    2 => format!("MOV R{}, R{}", rd, rs),
                    3 => format!("BX R{}", rs),
                    _ => "???".into(),
                }
            } else {
                let op = (opcode >> 9) & 0x7;
                let ro = (opcode >> 6) & 7;
                let rb = (opcode >> 3) & 7;
                let rd = opcode & 7;
                match op {
                    0 => format!("STR R{}, [R{}, R{}]", rd, rb, ro),
                    1 => format!("STRH R{}, [R{}, R{}]", rd, rb, ro),
                    2 => format!("STRB R{}, [R{}, R{}]", rd, rb, ro),
                    3 => format!("LDRSB R{}, [R{}, R{}]", rd, rb, ro),
                    4 => format!("LDR R{}, [R{}, R{}]", rd, rb, ro),
                    5 => format!("LDRH R{}, [R{}, R{}]", rd, rb, ro),
                    6 => format!("LDRB R{}, [R{}, R{}]", rd, rb, ro),
                    7 => format!("LDRSH R{}, [R{}, R{}]", rd, rb, ro),
                    _ => "???".into(),
                }
            }
        }
        0b011 => {
            let load = (opcode >> 11) & 1 != 0;
            let imm5 = ((opcode >> 6) & 0x1F) as u32;
            let rb = (opcode >> 3) & 7;
            let rd = opcode & 7;
            if load {
                format!("LDR R{}, [R{}, #{}]", rd, rb, imm5 << 2)
            } else {
                format!("STR R{}, [R{}, #{}]", rd, rb, imm5 << 2)
            }
        }
        0b100 => {
            let op = (opcode >> 11) & 0x3;
            let imm5 = ((opcode >> 6) & 0x1F) as u32;
            let rb = (opcode >> 3) & 7;
            let rd = opcode & 7;
            match op {
                0 => format!("STRH R{}, [R{}, #{}]", rd, rb, imm5 << 1),
                1 => format!("LDRH R{}, [R{}, #{}]", rd, rb, imm5 << 1),
                2 => format!("STR R{}, [R{}, #{}]", rd, rb, imm5 << 2),
                3 => format!("LDR R{}, [R{}, #{}]", rd, rb, imm5 << 2),
                _ => "???".into(),
            }
        }
        _ => format!("{:04X}", opcode),
    }
}
