use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.mem.vram_log_enabled = true;
    gba.mem.vram_write_log.clear();

    gba.run_frame_parallel(&mut fb);

    let n = gba.mem.vram_write_log.len();
    println!("Frame 200: {} VRAM writes", n);

    let mut pc_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &(_addr, pc, _val) in &gba.mem.vram_write_log {
        *pc_counts.entry(pc).or_insert(0) += 1;
    }

    let mut sorted: Vec<_> = pc_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));

    println!("Top 20 VRAM write PCs:");
    for (i, (pc, count)) in sorted.iter().take(20).enumerate() {
        println!("  {:2}: PC=0x{:08X}  count={}", i + 1, pc, count);
    }

    let text_writes: Vec<_> = gba
        .mem
        .vram_write_log
        .iter()
        .filter(|(addr, _, _)| *addr >= 0x06000000 && *addr < 0x06008000)
        .take(20)
        .collect();
    println!("\nFirst 20 VRAM writes to tile area (0x06000000-0x06008000):");
    for (addr, pc, val) in text_writes {
        println!("  addr=0x{:08X} pc=0x{:08X} val=0x{:02X}", addr, pc, val);
    }
}
