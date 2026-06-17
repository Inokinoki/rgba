use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Set a breakpoint-like trace: capture regs when PC hits 0x080D0900
    gba.mem.pc_trace_base = 0x080D0900;
    gba.mem.pc_trace_counts = vec![0; 0x200];

    // We need a custom approach - add a hook in the CPU
    // For now, let's use the IRQ trace mechanism but hijack it
    // Actually, let's just instrument the code differently

    // Run until we hit 0x080D0900 by checking after each step
    let mut hits = 0;
    let max_hits = 5;

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);

        // Check PC trace for hits
        let total_hits: u32 = gba.mem.pc_trace_counts.iter().sum();
        if total_hits > 0 && hits == 0 {
            println!(
                "Frame {}: Decompression code hit {} times",
                frame, total_hits
            );
        }
    }

    // That won't give us register state. Let me try a different approach.
    // We'll add a register trace field to Gba.
    println!("Need register trace mechanism");
}
