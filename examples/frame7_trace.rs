use rgba::Gba;
use std::collections::HashMap;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    // Run to frame 7 (stuck state)
    for _ in 0..7 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Now trace PCs for one frame
    let trace_base = 0x080D0000;
    let trace_size = 0x10000; // 64K halfwords = 128KB range
    gba.mem_mut().pc_trace_base = trace_base;
    gba.mem_mut().pc_trace_counts = vec![0u32; trace_size];

    gba.run_frame_parallel(&mut framebuffer);

    let counts = &gba.mem().pc_trace_counts;
    let mut hot_pcs: Vec<(u32, u32)> = Vec::new();
    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            hot_pcs.push((trace_base + (i as u32) * 2, count));
        }
    }
    hot_pcs.sort_by(|a, b| b.1.cmp(&a.1));

    println!("=== PC trace for frame 7 (stuck) ===");
    println!("Total unique PCs: {}", hot_pcs.len());
    let total: u32 = hot_pcs.iter().map(|(_, c)| *c).sum();
    println!("Total instructions: {}", total);

    println!("\nTop 40 PCs:");
    for (pc, count) in hot_pcs.iter().take(40) {
        println!("  {:#010X}: {} ({:.1}%)", pc, count, *count as f64 * 100.0 / total as f64);
    }

    // Group by function (assume functions are 0x10000 bytes apart)
    let mut func_counts: HashMap<u32, u32> = HashMap::new();
    for &(pc, count) in &hot_pcs {
        let func_base = pc & 0xFFFF0000;
        *func_counts.entry(func_base).or_insert(0) += count;
    }
    let mut funcs: Vec<_> = func_counts.into_iter().collect();
    funcs.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\nBy 64KB region:");
    for (base, count) in funcs.iter().take(10) {
        println!("  {:#010X}: {} instructions", base, count);
    }
}
