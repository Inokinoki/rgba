use rgba::Gba;
use std::collections::HashSet;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.mem_mut().swi_log_enabled = true;

    // Run 6 frames to get to the stuck state
    for _ in 0..6 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now trace unique PCs for frame 7
    let mut unique_pcs: HashSet<u32> = HashSet::new();
    let mut pc_seq: Vec<u32> = Vec::new();

    gba.mem_mut().pc_trace_base = 0x080D0000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x8000]; // 0x080D0000-0x080E0000

    gba.mem_mut().swi_log.clear();
    gba.run_frame_parallel(&mut fb);
    
    // Print PC trace counts
    let trace = &gba.mem().pc_trace_counts;
    println!("=== PC trace for frame 7 (0x080D0000-0x080E0000) ===");
    let mut hot_pcs: Vec<(u32, u32)> = trace.iter().enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| (0x080D0000 + (i as u32) * 2, c))
        .collect();
    hot_pcs.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("Total unique PCs: {} ({} with >0 hits)", 
        hot_pcs.len(), hot_pcs.len());
    println!("\nTop 50 hottest PCs:");
    for (pc, count) in hot_pcs.iter().take(50) {
        let thumb = if *pc & 1 != 0 { "T" } else { "A" };
        println!("  0x{:08X} ({}): {}", pc, thumb, count);
    }
    
    println!("\nAll PCs in execution order (first 200):");
    // Can't get sequence from counts, just print hot PCs
    for (pc, count) in hot_pcs.iter() {
        if *count > 100 {
            print!("0x{:08X} ", pc);
        }
    }
    println!();
    
    println!("\nSWIs: {:?}", gba.mem().swi_log);
    println!("PC=0x{:08X}", gba.cpu().get_pc());
    
    // Also trace IRQ handler execution
    let irq_trace = &gba.mem().irq_trace;
    if !irq_trace.is_empty() {
        println!("\nIRQ trace ({} entries):", irq_trace.len());
        for (i, entry) in irq_trace.iter().take(20).enumerate() {
            println!("  [{}] type={} pc/vcount=0x{:08X} IE=0x{:04X} IF=0x{:04X} halted={}",
                i, entry.0, entry.1, entry.2, entry.3, entry.4);
        }
    }
}
