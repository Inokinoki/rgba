use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.cpu_mut().decomp_trace_enabled = true;

    for frame in 0..200 {
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

    eprintln!("2nd tile call starts at entry {}", start2nd);

    let end2nd = trace
        .iter()
        .position(|&(_, _, regs)| regs[8] == 0x06007FFE && regs[1] > 0x0600FFFF)
        .unwrap_or(trace.len());

    eprintln!("2nd tile call r1 leaves VRAM at entry {}", end2nd);

    let before = end2nd.saturating_sub(5);
    let after = (end2nd + 20).min(trace.len());

    println!("\n=== Around transition at entry {} ===", end2nd);
    for i in before..after {
        let (pc, opcode, regs) = &trace[i];
        let r1 = regs[1];
        let r8 = regs[8];
        let r0 = regs[0];
        let r2 = regs[2];
        let r3 = regs[3];
        let r4 = regs[4];
        let r6 = regs[6];
        let r10 = regs[10];
        println!("  @{}: pc={:08X} op={:04X} r0={:08X} r1={:08X} r2={:08X} r3={:08X} r4={:08X} r6={:08X} r8={:08X} r10={:08X}",
            i, pc, *opcode as u16, r0, r1, r2, r3, r4, r6, r8, r10);
    }

    let last_vram = trace[..end2nd]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, (_, _, regs))| {
            regs[1] >= 0x06000000 && regs[1] < 0x0600C000 && regs[8] == 0x06007FFE
        });

    if let Some((idx, (pc, opcode, regs))) = last_vram {
        println!(
            "\n=== Last tile-area entry @{}: pc={:08X} r1={:08X} r8={:08X} r0={:08X} ===",
            idx, pc, regs[1], regs[8], regs[0]
        );

        let start = idx.saturating_sub(3);
        let end = (idx + 30).min(trace.len());
        println!("\n=== Trace from {} to {} ===", start, end);
        for i in start..end {
            let (pc, opcode, regs) = &trace[i];
            let r1 = regs[1];
            let in_tile = r1 >= 0x06000000 && r1 < 0x0600C000;
            let marker = if in_tile { "" } else { " <<<" };
            println!("  @{}: pc={:08X} op={:04X} r0={:08X} r1={:08X} r3={:08X} r4={:08X} r6={:08X} r8={:08X}{}",
                i, pc, *opcode as u16, regs[0], r1, regs[3], regs[4], regs[6], regs[8], marker);
        }
    }
}
