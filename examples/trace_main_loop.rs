use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    
    // Run 6 frames to get to stuck state
    for _ in 0..6 { gba.run_frame_parallel(&mut fb); }

    // Now trace ALL PCs for one frame (wider range)
    gba.mem_mut().pc_trace_base = 0x080C0000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x20000]; // 0x080C0000-0x08100000

    gba.run_frame_parallel(&mut fb);
    
    // Print hot PCs grouped by function
    let trace = &gba.mem().pc_trace_counts;
    let mut hot_pcs: Vec<(u32, u32)> = trace.iter().enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| (0x08000000 + (i as u32) * 2, c))
        .collect();
    hot_pcs.sort_by_key(|&(pc, _)| pc);
    
    println!("=== All hot PCs by region (count > 50) ===");
    let mut current_region = 0u32;
    let mut region_total = 0u32;
    for &(pc, count) in &hot_pcs {
        let region_base = pc & 0xFFFFF000; // 4K regions
        if region_base != current_region {
            if region_total > 0 {
                println!("  Region 0x{:08X}: {} total hits", current_region, region_total);
            }
            current_region = region_base;
            region_total = 0;
        }
        region_total += count;
        if count > 50 {
            println!("  0x{:08X}: {}", pc, count);
        }
    }
    if region_total > 0 {
        println!("  Region 0x{:08X}: {} total hits", current_region, region_total);
    }
    
    // Find the code that calls SWI 4 (IntrWait wrapper at 0x080D2F0C)
    // Look for BL to 0x080D2F0C
    println!("\n=== Looking for calls to IntrWait wrapper ===");
    // Search for BL instructions that target 0x080D2F0C
    for &(pc, _) in &hot_pcs {
        if pc >= 0x080D2F00 && pc <= 0x080D2F20 {
            println!("  PC at 0x{:08X}: {}", pc, trace[((pc - 0x08000000) / 2) as usize]);
        }
    }
}
