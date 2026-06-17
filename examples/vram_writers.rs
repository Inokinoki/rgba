use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();
    gba.mem_mut().pc_trace_base = 0x080D0000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x4000];

    // Run for 300 frames
    for _ in 0..300 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    // Show which ROM addresses wrote to VRAM BG tile area
    let log = &gba.mem().vram_write_log;
    let bg_writes: Vec<_> = log
        .iter()
        .filter(|(addr, _, _)| *addr >= 0x06000000 && *addr < 0x0600F000)
        .collect();

    println!("BG tile area writes: {}", bg_writes.len());

    // Group by PC
    let mut pc_groups: std::collections::BTreeMap<u32, Vec<(u32, u8)>> =
        std::collections::BTreeMap::new();
    for &&(addr, pc, val) in &bg_writes {
        pc_groups.entry(pc).or_default().push((addr, val));
    }

    println!("\nPCs writing to BG tiles:");
    for (pc, writes) in &pc_groups {
        let addrs: Vec<u32> = writes.iter().map(|(a, _)| *a).collect();
        let min_addr = addrs.iter().min().unwrap();
        let max_addr = addrs.iter().max().unwrap();
        let unique_vals: std::collections::HashSet<u8> = writes.iter().map(|(_, v)| *v).collect();
        println!(
            "  PC={:08X}: {} writes, addr range {:08X}-{:08X}, {} unique values",
            pc,
            writes.len(),
            min_addr,
            max_addr,
            unique_vals.len()
        );
    }

    // Show the PC trace for the 0x080D0000-0x080DFFFF range
    let trace = &gba.mem().pc_trace_counts;
    let total: u32 = trace.iter().sum();
    println!("\nPC trace for 0x080D0000-0x080DFFFF: {} total hits", total);

    // Show hotspots
    let mut hotspots: Vec<(usize, u32)> = trace
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| (i, c))
        .collect();
    hotspots.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Top 20 hotspots:");
    for (offset, count) in hotspots.iter().take(20) {
        let addr = 0x080D0000 + offset;
        println!("  {:08X}: {} executions", addr, count);
    }
}
