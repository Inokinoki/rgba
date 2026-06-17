use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    println!("=== PC trace in loading function range (0x080D0900-0x080D0CB0) ===");
    gba.mem_mut().pc_trace_base = 0x080D0900;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x3D0 / 2];

    for frame in 0..260u32 {
        let prev_sum: u64 = gba.mem().pc_trace_counts.iter().map(|&c| c as u64).sum();
        gba.run_frame_parallel(&mut framebuffer);
        let after_sum: u64 = gba.mem().pc_trace_counts.iter().map(|&c| c as u64).sum();
        let delta = after_sum - prev_sum;

        if delta > 0 {
            let vram = gba.mem().vram();
            let nonzero = vram.iter().filter(|&&b| b != 0).count();
            println!(
                "Frame {}: {} hits in loading range, VRAM nonzero={}",
                frame, delta, nonzero
            );
        }
    }

    println!("\n=== Hot PCs in loading function ===");
    let counts = &gba.mem().pc_trace_counts;
    let base = gba.mem().pc_trace_base;
    let mut hot: Vec<(u32, u32)> = Vec::new();
    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            let pc = base + (i as u32) * 2;
            hot.push((pc, count));
        }
    }
    hot.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
    for (pc, count) in hot.iter().take(30) {
        println!("  {:#010X}: {} hits", pc, count);
    }
}
