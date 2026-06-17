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
        if frame % 50 == 0 {
            eprintln!("Frame {}", frame);
        }
    }

    let trace = &gba.cpu().decomp_trace;
    eprintln!("Total trace entries: {}", trace.len());

    let mut r1_targets: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    let mut r8_targets: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();

    for &(pc, _opcode, regs) in trace.iter() {
        let r1 = regs[1];
        let r8 = regs[8];

        let r1_region = if r1 >= 0x06000000 && r1 < 0x0600C000 {
            "VRAM-tile"
        } else if r1 >= 0x0600C000 && r1 < 0x06010000 {
            "VRAM-screen"
        } else if r1 >= 0x02000000 && r1 < 0x02040000 {
            "EWRAM"
        } else if r1 >= 0x03000000 && r1 < 0x03008000 {
            "IWRAM"
        } else {
            "OTHER"
        };

        let r8_region = if r8 >= 0x06000000 && r8 < 0x0600C000 {
            "VRAM-tile"
        } else if r8 >= 0x0600C000 && r8 < 0x06010000 {
            "VRAM-screen"
        } else if r8 >= 0x02000000 && r8 < 0x02040000 {
            "EWRAM"
        } else if r8 >= 0x03000000 && r8 < 0x03008000 {
            "IWRAM"
        } else {
            "OTHER"
        };

        *r1_targets
            .entry(format!("{} {:08X}", r1_region, r1 & 0xFFFF0000))
            .or_insert(0) += 1;
        *r8_targets
            .entry(format!("{} {:08X}", r8_region, r8 & 0xFFFF0000))
            .or_insert(0) += 1;
    }

    println!("\nr1 destination distribution:");
    for (region, count) in &r1_targets {
        println!("  {}: {} entries", region, count);
    }

    println!("\nr8 limit distribution:");
    for (region, count) in &r8_targets {
        println!("  {}: {} entries", region, count);
    }

    let mut r1_vram_entries: Vec<(usize, u32, u32, u32)> = Vec::new();
    for (i, &(pc, _opcode, regs)) in trace.iter().enumerate() {
        if regs[1] >= 0x06000000 && regs[1] < 0x06010000 {
            if r1_vram_entries.is_empty() || (i - r1_vram_entries.last().unwrap().0) > 100 {
                r1_vram_entries.push((i, pc, regs[1], regs[8]));
            }
        }
    }

    println!("\nr1 in VRAM transitions (sampled):");
    for (i, pc, r1, r8) in r1_vram_entries.iter().take(30) {
        println!("  entry {}: pc={:08X} r1={:08X} r8={:08X}", i, pc, r1, r8);
    }

    let mut r1_vram_tile: Vec<(usize, u32, u32)> = Vec::new();
    for (i, &(pc, _opcode, regs)) in trace.iter().enumerate() {
        if regs[1] >= 0x06000000 && regs[1] < 0x0600C000 {
            if r1_vram_tile.is_empty() || regs[1] < r1_vram_tile.last().unwrap().2 - 0x100 {
                r1_vram_tile.push((i, pc, regs[1]));
            }
        }
    }

    println!("\nr1 in VRAM TILE area (key transitions):");
    for (i, pc, r1) in r1_vram_tile.iter().take(30) {
        println!("  entry {}: pc={:08X} r1={:08X}", i, pc, r1);
    }
}
