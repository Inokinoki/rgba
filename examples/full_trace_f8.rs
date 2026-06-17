use rgba::Gba;
use std::collections::HashMap;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    // Run to frame 8 (well into stuck state)
    for _ in 0..8 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Use a list-based trace for one frame to capture ALL PCs
    // We'll instrument by using the pc_trace mechanism but with BIOS+IWRAM+ROM coverage
    // Since pc_trace uses a single contiguous range, let's use two: ROM and IWRAM/BIOS

    // Trace ROM range
    gba.mem_mut().pc_trace_base = 0x00000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x04000000 / 2]; // 0 to 0x04000000 (covers BIOS, IWRAM, ROM)

    gba.run_frame_parallel(&mut framebuffer);

    let counts = &gba.mem().pc_trace_counts;
    let base = 0u32;

    // Find all unique PCs
    let mut hot_pcs: Vec<(u32, u32)> = Vec::new();
    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            hot_pcs.push((base + (i as u32) * 2, count));
        }
    }

    // Sort by address (not by count) to see execution flow
    hot_pcs.sort_by_key(|(pc, _)| *pc);

    println!("=== All PCs executed in frame 8 (sorted by address) ===");
    println!("Total unique PCs: {}", hot_pcs.len());
    let total: u32 = hot_pcs.iter().map(|(_, c)| *c).sum();
    println!("Total instructions: {}", total);

    // Group by region
    let mut prev_region = "";
    for &(pc, count) in &hot_pcs {
        let region = if pc < 0x4000 {
            "BIOS"
        } else if pc >= 0x02000000 && pc < 0x03000000 {
            "EWRAM"
        } else if pc >= 0x03000000 && pc < 0x04000000 {
            "IWRAM"
        } else if pc >= 0x08000000 {
            "ROM"
        } else {
            "OTHER"
        };

        if region != prev_region {
            println!("\n--- {} ---", region);
            prev_region = region;
        }

        // Only print if count > 0 (already filtered) and in regions of interest
        if pc < 0x4000 || (pc >= 0x03000000 && pc < 0x03010000) {
            println!("  {:#010X}: {}", pc, count);
        }
    }

    // Print ROM PCs sorted by count
    let rom_pcs: Vec<_> = hot_pcs.iter().filter(|(pc, _)| *pc >= 0x08000000).collect();
    let mut rom_by_count = rom_pcs.clone();
    rom_by_count.sort_by(|a, b| b.1.cmp(&a.1));
    println!("\n--- ROM (top 20 by count) ---");
    for &(pc, count) in rom_by_count.iter().take(20) {
        println!("  {:#010X}: {}", pc, count);
    }
}
