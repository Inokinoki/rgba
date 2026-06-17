use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..195 { gba.run_frame_parallel(&mut fb); }

    // Trace frame 195 - the stuck frame
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x100000]; // 0x08000000-0x08200000

    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().swi_log.clear();
    
    gba.run_frame_parallel(&mut fb);
    
    let trace = &gba.mem().pc_trace_counts;
    
    // Group by 4K regions
    let mut regions: Vec<(u32, u32)> = Vec::new();
    for i in 0..trace.len() {
        if trace[i] > 0 {
            let pc = 0x08000000 + (i as u32) * 2;
            let region = pc & 0xFFFFF000;
            if let Some(last) = regions.last_mut() {
                if last.0 == region {
                    last.1 += trace[i];
                } else {
                    regions.push((region, trace[i]));
                }
            } else {
                regions.push((region, trace[i]));
            }
        }
    }
    
    println!("=== Region summary for stuck frame ===");
    for (region, total) in &regions {
        if *total > 10 {
            println!("0x{:08X}: {} hits", region, total);
        }
    }
    
    // Show top individual PCs
    let mut hot: Vec<(u32, u32)> = trace.iter().enumerate()
        .filter(|(_, &c)| c > 5)
        .map(|(i, &c)| (0x08000000 + (i as u32) * 2, c))
        .collect();
    hot.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("\n=== Top 30 individual PCs ===");
    for (pc, count) in hot.iter().take(30) {
        println!("0x{:08X}: {}", pc, count);
    }
    
    println!("\nSWIs: {:?}", gba.mem().swi_log);
    println!("PC=0x{:08X}", gba.cpu().get_pc());
}
