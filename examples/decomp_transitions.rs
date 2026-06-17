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

    let mut last_r8 = 0xFFFFFFFFu32;
    let mut last_r1_region = String::new();

    for (i, &(pc, _opcode, regs)) in trace.iter().enumerate() {
        let r1 = regs[1];
        let r8 = regs[8];

        let r1_region = if r1 >= 0x06000000 && r1 < 0x0600C000 {
            "TILE"
        } else if r1 >= 0x0600C000 && r1 < 0x06010000 {
            "SCR"
        } else if r1 >= 0x02000000 && r1 < 0x02040000 {
            "EWRAM"
        } else if r1 >= 0x03000000 && r1 < 0x03008000 {
            "IWRAM"
        } else {
            "OTHER"
        };

        let r8_region = if r8 >= 0x06000000 && r8 < 0x0600C000 {
            "TILE"
        } else if r8 >= 0x0600C000 && r8 < 0x06010000 {
            "SCR"
        } else if r8 >= 0x02000000 && r8 < 0x02040000 {
            "EWRAM"
        } else if r8 >= 0x03000000 && r8 < 0x03008000 {
            "IWRAM"
        } else {
            "OTHER"
        };

        if r8 != last_r8 || r1_region != last_r1_region {
            let r1_end = if i > 0 { trace[i - 1].2[1] } else { 0 };
            let written = if last_r8 != 0xFFFFFFFF && i > 0 {
                r1_end.wrapping_sub(trace[0].2[1])
            } else {
                0
            };

            println!(
                "  @{}: r1={:08X}({}) r8={:08X}({}) r0={:08X} r3={:08X} r9={:08X}",
                i, r1, r1_region, r8, r8_region, regs[0], regs[3], regs[9]
            );
            last_r8 = r8;
            last_r1_region = r1_region.to_string();
        }
    }

    println!("\n=== Last 10 tile-area entries ===");
    let tile_entries: Vec<_> = trace
        .iter()
        .enumerate()
        .filter(|(_, (_, _, regs))| regs[1] >= 0x06000000 && regs[1] < 0x0600C000)
        .collect();
    for (i, (pc, _, regs)) in tile_entries.iter().rev().take(10) {
        println!(
            "  @{}: pc={:08X} r1={:08X} r8={:08X}",
            i, pc, regs[1], regs[8]
        );
    }

    println!("\n=== First 10 screen-area entries ===");
    let scr_entries: Vec<_> = trace
        .iter()
        .enumerate()
        .filter(|(_, (_, _, regs))| regs[1] >= 0x0600C000 && regs[1] < 0x06010000)
        .collect();
    for (i, (pc, _, regs)) in scr_entries.iter().take(10) {
        println!(
            "  @{}: pc={:08X} r1={:08X} r8={:08X}",
            i, pc, regs[1], regs[8]
        );
    }
}
