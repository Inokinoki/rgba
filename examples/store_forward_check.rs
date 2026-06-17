use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

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

    // STRH R4,[R1,#0] opcodes in THUMB:
    // 0x8001 = STRH R0,[R0,R1] - no
    // STRH Rd,[Rb,Ro] format: 0101 001 Ro Rb Rd
    // STRH R4,[R1,#0] = STRH Rd,[Rb] with Ro=R0? No, this is STRH with register offset
    // 0x8019 = STRH R4,[R3, R0]? Let me check
    // Actually: STRH Rd,[Rb,Ro] = 0101 001 | Ro(3) | Rb(3) | Rd(3)
    // STRH R4,[R1,R0] = 0101 001 | 000 | 001 | 100 = 0101 0010 0001 1100 = 0x521C
    // STRH R4,[R1] with Ro=0: 0101 001 | 000 | 001 | 100 = 0x521C

    // Let me find the actual opcodes at the key addresses
    // 0x080D0BEA: STRH R4,[R1,#0] - this is THUMB format 5 STRH with immediate offset 0
    // Actually, in THUMB: STRH Rd,[Rb,#imm] format = 1000 0 imm5 Rb Rd
    // STRH R4,[R1,#0] = 1000 0 00000 001 100 = 1000 0000 0000 1100 = 0x800C
    //
    // Wait, let me check. The trace has (pc, opcode, registers).
    // Let me just find entries where STRH/LDRH happen with R1 as base and analyze.

    // First, let's find all STRH instructions in the trace that write to IWRAM range
    // The key addresses are:
    // 0x080D0BEA: literal path STRH
    // 0x080D0BFA: copy path STRH
    // 0x080D0BF4: copy path LDRH

    let target_iwram = 0x03006DD8u32;

    let mut literal_writes: Vec<(usize, u32, u16, u32)> = Vec::new(); // (trace_idx, addr, value, r4)
    let mut copy_writes: Vec<(usize, u32, u16, u32, u32)> = Vec::new(); // (trace_idx, addr, value_from_r4, r1, r5)
    let mut copy_reads: Vec<(usize, u32, u32, u32)> = Vec::new(); // (trace_idx, addr, r5_value, r4_before)

    for (i, &(pc, opcode, regs)) in trace.iter().enumerate() {
        let abs_pc = pc & !1;

        // Literal STRH: 0x080D0BEA - writes R4 to [R1]
        if abs_pc == 0x080D0BEA {
            let addr = regs[1];
            let val = regs[4] as u16;
            if addr >= target_iwram && addr < target_iwram + 0x200 {
                literal_writes.push((i, addr, val, regs[4]));
            }
        }

        // Copy LDRH: 0x080D0BF4 - reads from [R5] into R4
        if abs_pc == 0x080D0BF4 {
            let addr = regs[5];
            if addr >= target_iwram && addr < target_iwram + 0x200 {
                copy_reads.push((i, addr, regs[5], regs[4]));
            }
        }

        // Copy STRH: 0x080D0BFA - writes R4 to [R1]
        if abs_pc == 0x080D0BFA {
            let addr = regs[1];
            let val = regs[4] as u16;
            if addr >= target_iwram && addr < target_iwram + 0x200 {
                copy_writes.push((i, addr, val, regs[1], regs[5]));
            }
        }
    }

    println!(
        "Literal STRH writes to IWRAM target: {}",
        literal_writes.len()
    );
    println!("Copy LDRH reads from IWRAM target: {}", copy_reads.len());
    println!("Copy STRH writes to IWRAM target: {}", copy_writes.len());

    // Show first literal writes
    println!("\n=== First literal writes (STRH R4,[R1] at 0x080D0BEA) ===");
    for (i, addr, val, r4) in literal_writes.iter().take(40) {
        let offset = addr - target_iwram;
        let tile_id = val & 0x3FF;
        let palette = (val >> 12) & 0xF;
        println!(
            "  [{:6}] addr={:08X} (+{:03X}) val={:04X} (tile={} pal={}) r4={:08X}",
            i, addr, offset, val, tile_id, palette, r4
        );
    }

    // Show first copy reads
    println!("\n=== First copy reads (LDRH from R5 at 0x080D0BF4) ===");
    for (i, addr, r5, r4_before) in copy_reads.iter().take(40) {
        let offset = addr - target_iwram;
        println!(
            "  [{:6}] addr={:08X} (+{:03X}) r5={:08X} r4_before={:08X}",
            i, addr, offset, r5, r4_before
        );
    }

    // Show first copy writes
    println!("\n=== First copy writes (STRH R4,[R1] at 0x080D0BFA) ===");
    for (i, addr, val, r1, r5) in copy_writes.iter().take(40) {
        let offset = addr - target_iwram;
        let tile_id = val & 0x3FF;
        let palette = (val >> 12) & 0xF;
        println!(
            "  [{:6}] addr={:08X} (+{:03X}) val={:04X} (tile={} pal={}) r1={:08X} r5={:08X}",
            i, addr, offset, val, tile_id, palette, r1, r5
        );
    }

    // Now the key check: for each copy write, what was the source address?
    // The copy path reads from r5, then writes to r1
    // r5 = output_ptr - distance (back-reference)
    // We need to check: was the value at [r5] ever written by a previous STRH?

    println!("\n=== Store-forward consistency check ===");
    let mut inconsistencies = 0u32;
    let mut consistent = 0u32;

    // Build a map: addr -> last written value (from literal and copy writes)
    let mut write_history: std::collections::BTreeMap<u32, Vec<(usize, u16)>> =
        std::collections::BTreeMap::new();

    // Process in trace order
    for (i, &(pc, opcode, regs)) in trace.iter().enumerate() {
        let abs_pc = pc & !1;

        // Record writes
        if abs_pc == 0x080D0BEA || abs_pc == 0x080D0BFA {
            let addr = regs[1] & !1;
            let val = regs[4] as u16;
            if addr >= target_iwram && addr < target_iwram + 0x200 {
                write_history.entry(addr).or_default().push((i, val));
            }
        }

        // Check reads in copy path
        if abs_pc == 0x080D0BF4 {
            let read_addr = regs[5] & !1;
            if read_addr >= target_iwram && read_addr < target_iwram + 0x200 {
                // What was the last value written to this address before this read?
                if let Some(writes) = write_history.get(&read_addr) {
                    if let Some(&(_, last_val)) = writes.last() {
                        // The value that will be read should match what was written
                        // But we don't know the actual read value from the trace (only regs BEFORE the instruction)
                        // We can check: the next instruction's R4 should be the value read
                        if i + 1 < trace.len() {
                            let r4_after = trace[i + 1].2[4];
                            let r4_low = r4_after as u16;
                            if r4_low != last_val && inconsistencies < 20 {
                                println!(
                                    "  INCONSISTENCY at trace[{}]: read from {:08X}, last written={:04X}, but R4 after={:04X}",
                                    i, read_addr, last_val, r4_low
                                );
                                inconsistencies += 1;
                            } else if r4_low == last_val {
                                consistent += 1;
                            }
                        }
                    }
                } else if inconsistencies < 20 {
                    println!(
                        "  READ FROM UNWRITTEN addr at trace[{}]: {:08X} (never written before)",
                        i, read_addr
                    );
                    inconsistencies += 1;
                }
            }
        }
    }

    println!(
        "\nStore-forward: {} consistent, {} inconsistent",
        consistent, inconsistencies
    );

    // Final check: what's in IWRAM now?
    let iwram = gba.mem().iwram();
    println!("\n=== IWRAM at 0x03006DD8 (current state) ===");
    for i in 0..64 {
        let offset = 0x6DD8 + i * 2;
        let val = u16::from_le_bytes([iwram[offset], iwram[offset + 1]]);
        let tile_id = val & 0x3FF;
        let palette = (val >> 12) & 0xF;
        if i < 10 || palette != 11 {
            println!(
                "  [{:2}] {:08X}: {:04X} (tile={} pal={})",
                i,
                0x03006DD8 + i * 2,
                val,
                tile_id,
                palette
            );
        }
    }
}
