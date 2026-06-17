use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..170 {
        for _sl in 0..228 {
            gba.run_scanline();
        }
    }

    gba.cpu_mut().decomp_trace_enabled = true;

    for frame in 0..30 {
        for _sl in 0..228 {
            gba.run_scanline();
        }
    }

    let trace = &gba.cpu().decomp_trace;
    eprintln!("Total: {}", trace.len());

    let start2nd = trace
        .iter()
        .position(|&(_, _, regs)| {
            regs[8] == 0x06007FFE && regs[1] >= 0x06000000 && regs[1] < 0x0600C000
        })
        .unwrap();

    let end2nd = trace
        .iter()
        .position(|&(_, _, regs)| regs[8] == 0x06007FFE && regs[1] > 0x0600FFFF)
        .unwrap_or(trace.len());

    eprintln!("2nd tile call: {} to {}", start2nd, end2nd);

    let bx_idx = trace[..end2nd]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, (pc, op, regs))| {
            let abs_pc = *pc & !1;
            let opcode = *op as u16;
            abs_pc == 0x080D0BDC
                && opcode == 0x4748
                && regs[1] >= 0x06000000
                && regs[1] < 0x0600C000
        });

    if let Some((bx_entry, _)) = bx_idx {
        eprintln!("Last BX r9 with valid r1 at entry {}", bx_entry);

        let start = bx_entry;
        let end = (bx_entry + 80).min(trace.len());

        println!("\n=== Trace from BX r9 onward ===");
        for i in start..end {
            let (pc, opcode, regs) = &trace[i];
            let abs_pc = *pc & !1;
            let op16 = *opcode as u16;
            let r0 = regs[0];
            let r1 = regs[1];
            let r2 = regs[2];
            let r3 = regs[3];
            let r4 = regs[4];
            let r7 = regs[7];
            let r14 = regs[14];
            let marker = if abs_pc >= 0x080D0920 && abs_pc < 0x080D0960 {
                " [HELPER]"
            } else {
                ""
            };
            let r1_bad = if r1 >= 0x02000000 && r1 < 0x03000000 && i > bx_entry {
                " <<<R1_CORRUPT"
            } else {
                ""
            };
            println!(
                "  @{}: pc={:08X} op={:04X} r0={:08X} r1={:08X} r2={:08X} r3={:08X} r4={:08X} r7={:08X} lr={:08X}{}{}",
                i, pc, op16, r0, r1, r2, r3, r4, r7, r14, marker, r1_bad
            );
        }
    }
}
