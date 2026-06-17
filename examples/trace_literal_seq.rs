use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.cpu_mut().decomp_trace_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];
    for frame in 0..300 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let trace = &gba.cpu().decomp_trace;

    // Find first 5 literal STRH writes
    let mut literal_indices: Vec<usize> = Vec::new();
    for (i, &(pc, _opcode, regs)) in trace.iter().enumerate() {
        let abs_pc = pc & !1;
        if abs_pc == 0x080D0BEA && regs[1] >= 0x03006DD8 && regs[1] < 0x03007000 {
            literal_indices.push(i);
            if literal_indices.len() >= 5 {
                break;
            }
        }
    }

    // Show the complete sequence between literal #0 and literal #2
    let start = literal_indices[0];
    let end = literal_indices[2];

    println!("=== Complete trace from literal #0 through literal #2 ===");
    println!("(trace entries {} to {})\n", start, end);

    for j in start..=end {
        let (pc, opcode, regs) = trace[j];
        let abs_pc = pc & !1;
        let thumb = pc & 1 != 0;
        let opc = if thumb { opcode as u16 } else { opcode as u16 };

        print!("[{:7}] {:08X}: {:04X}  ", j, abs_pc, opc);

        // Show key registers
        print!(
            "R0={:08X} R1={:08X} R2={:08X} R3={:02X} R4={:08X} R5={:08X} R6={:08X}",
            regs[0], regs[1], regs[2], regs[3], regs[4], regs[5], regs[6]
        );

        // Annotate key instructions
        if abs_pc == 0x080D0BEA {
            let val = regs[4] as u16;
            let tile = val & 0x3FF;
            let pal = (val >> 12) & 0xF;
            print!(
                " ; <<< STRH R4={:04X} (tile={} pal={}) to [R1={:08X}]",
                val, tile, pal, regs[1]
            );
        }
        if abs_pc == 0x080D0BE4 {
            print!(
                " ; LSL R4, R4, #8 (R4 {:08X} -> should become {:08X})",
                regs[4],
                regs[4] << 8
            );
        }
        if abs_pc == 0x080D0BE6 {
            print!(" ; ADDS R4, R4, R5");
        }
        if abs_pc == 0x080D0932 {
            print!(" ; ORR R4, R6 (byte decode: R4 | R6)");
        }
        if abs_pc == 0x080D0934 {
            print!(" ; POP {{R4,R5,R6,PC}} return from huffman");
        }
        if abs_pc == 0x080D0954 {
            print!(" ; BX R6 (huffman return)");
        }

        println!();
    }
}
