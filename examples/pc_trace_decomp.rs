use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Set up PC trace around the decompression range
    gba.mem.pc_trace_base = 0x080D0000;
    gba.mem.pc_trace_counts = vec![0u32; 0x2000]; // 0x4000 bytes = 0x2000 halfwords

    for frame in 0..200u32 {
        gba.run_frame_parallel(&mut fb);

        if frame % 50 == 0 || frame == 2 {
            // Check if any PC in the decompression range was hit
            let decomp_start = (0x080D0900 - 0x080D0000) / 2;
            let decomp_end = (0x080D0C20 - 0x080D0000) / 2;
            let mut decomp_hits = 0u32;
            for i in decomp_start..decomp_end {
                decomp_hits += gba.mem.pc_trace_counts[i];
            }

            // Check broader range around it
            let mut total_hits = 0u32;
            for i in 0..gba.mem.pc_trace_counts.len() {
                total_hits += gba.mem.pc_trace_counts[i];
            }

            // Find hottest PCs in the trace
            let mut hot: Vec<(usize, u32)> = gba
                .mem
                .pc_trace_counts
                .iter()
                .enumerate()
                .filter(|(_, &c)| c > 0)
                .map(|(i, &c)| (i, c))
                .collect();
            hot.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

            println!(
                "Frame {:4}: decomp_hits={} total_D000-D400={}",
                frame, decomp_hits, total_hits
            );
            if frame <= 5 {
                for (idx, count) in hot.iter().take(15) {
                    let pc = 0x080D0000 + idx * 2;
                    println!("  {:08X}: {} hits", pc, count);
                }
            }
        }
    }
}
