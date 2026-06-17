use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().vram_write_log;

    let mut pc_counts: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for (_addr, pc, _val) in log {
        let offset = (pc - 0x08000000) as usize;
        *pc_counts.entry(offset as u32).or_insert(0) += 1;
    }

    let mut sorted: Vec<_> = pc_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("Top 20 PCs that write to VRAM:");
    for (offset, count) in sorted.iter().take(20) {
        let pc = 0x08000000 + *offset;
        println!("  PC={:#010X} count={}", pc, count);
    }
}
