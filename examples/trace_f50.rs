use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..50 { gba.run_frame_parallel(&mut fb); }

    // Trace frame 50
    gba.mem_mut().pc_trace_base = 0x080C0000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x20000];

    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().swi_log.clear();
    
    gba.run_frame_parallel(&mut fb);
    
    let trace = &gba.mem().pc_trace_counts;
    let mut hot: Vec<(u32, u32)> = trace.iter().enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| (0x080C0000 + (i as u32) * 2, c))
        .collect();
    hot.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("=== Frame 50 hot PCs ===");
    let total: u32 = hot.iter().map(|(_, c)| *c).sum();
    println!("Total PC hits: {}", total);
    
    println!("\nTop 20:");
    for (pc, count) in hot.iter().take(20) {
        println!("0x{:08X}: {}", pc, count);
    }
    
    println!("\nSWIs: {:?}", gba.mem().swi_log);
    
    // Count how many cycles the CPU was halted vs running
    // If most of the frame is halt, total PC hits should be low
}
