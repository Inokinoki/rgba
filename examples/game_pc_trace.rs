use rgba::Gba;

fn main() {
    let mut gba = Gba::new();

    // Trace execution in the 0x080D0000-0x080D1000 range (4KB = 2048 halfwords)
    gba.mem.pc_trace_base = 0x080D0000;
    gba.mem.pc_trace_counts = vec![0u32; 0x800]; // 2048 entries for 4KB range

    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..600 {
        gba.run_frame();
    }

    let counts = &gba.mem.pc_trace_counts;
    println!("PC histogram for 0x080D0000-0x080D1000 after 600 frames:");
    println!("(showing addresses with > 0 executions)");
    println!();

    let mut total_execs = 0u64;
    let mut min_addr = 0xFFFFu32;
    let mut max_addr = 0u32;

    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            let addr = 0x080D0000 + (i as u32) * 2;
            total_execs += count as u64;
            min_addr = min_addr.min(addr);
            max_addr = max_addr.max(addr);
            println!("{:#010X}: {}", addr, count);
        }
    }

    println!("\nTotal executions in range: {}", total_execs);
    if min_addr <= max_addr {
        println!(
            "Address range with executions: {:#010X}-{:#010X}",
            min_addr, max_addr
        );
    }

    // Also trace the game's main code area: 0x08008000-0x0800A000
    // Run another 100 frames with this range
    let mut gba2 = Gba::new();
    gba2.mem.pc_trace_base = 0x08008000;
    gba2.mem.pc_trace_counts = vec![0u32; 0x1000]; // 8KB = 4096 halfwords
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..600 {
        gba2.run_frame();
    }

    let counts2 = &gba2.mem.pc_trace_counts;
    println!("\n\nPC histogram for 0x08008000-0x0800A000 after 600 frames:");

    let mut total2 = 0u64;
    let mut ranges = Vec::new();
    let mut range_start = 0u32;
    let mut range_end = 0u32;
    let mut in_range = false;

    for (i, &count) in counts2.iter().enumerate() {
        let addr = 0x08008000 + (i as u32) * 2;
        if count > 0 {
            total2 += count as u64;
            if !in_range {
                range_start = addr;
                in_range = true;
            }
            range_end = addr;
        } else {
            if in_range {
                ranges.push((range_start, range_end));
                in_range = false;
            }
        }
    }
    if in_range {
        ranges.push((range_start, range_end));
    }

    println!("Execution regions:");
    for (s, e) in &ranges {
        let size = e - s + 2;
        let mut total_in_range = 0u64;
        for (i, &c) in counts2.iter().enumerate() {
            let a = 0x08008000 + (i as u32) * 2;
            if a >= *s && a <= *e {
                total_in_range += c as u64;
            }
        }
        println!(
            "  {:#010X}-{:#010X} ({} bytes): {} execs",
            s, e, size, total_in_range
        );
    }

    // Print individual addresses for the 0x08008E00-0x08009000 region (where tile 1023 write happens)
    println!("\nDetail around tile 1023 write code (0x08008E00-0x08009000):");
    for (i, &count) in counts2.iter().enumerate() {
        let addr = 0x08008000 + (i as u32) * 2;
        if addr >= 0x08008E00 && addr < 0x08009000 && count > 0 {
            println!("  {:#010X}: {}", addr, count);
        }
    }
}
