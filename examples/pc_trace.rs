use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let pc_trace = &gba.mem().pc_trace_counts;
    let base = gba.mem().pc_trace_base;

    println!("PC trace base: {:#010X}", base);
    println!("PC trace len: {}", pc_trace.len());

    if !pc_trace.is_empty() {
        let mut sorted: Vec<(usize, u32)> = pc_trace
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(i, &c)| (i, c))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\nTop 30 PCs:");
        for (i, (idx, count)) in sorted.iter().take(30).enumerate() {
            let pc = base + (*idx as u32) * 4;
            println!("  {}: PC={:#010X} count={}", i + 1, pc, count);
        }
    }
}
