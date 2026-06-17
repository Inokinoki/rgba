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

    gba.mem.timer_writes_enabled = true;

    println!("=== Timer write counts per frame ===");
    for f in 0..20 {
        gba.mem.timer_writes.clear();
        let v_before = gba.mem.read_word(0x02000050);
        gba.run_frame_parallel(&mut fb);
        let v_after = gba.mem.read_word(0x02000050);
        let n = gba.mem.timer_writes.len();

        // Count unique PCs
        let mut pcs: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for &(addr, pc, val) in &gba.mem.timer_writes {
            pcs.insert(pc);
        }

        if n > 0 {
            println!(
                "Frame {}: [50]={:08X}->{:08X}  writes={} unique_pcs={:?}",
                200 + f,
                v_before,
                v_after,
                n,
                pcs
            );
            for &(addr, pc, val) in &gba.mem.timer_writes.iter().take(10).collect::<Vec<_>>() {
                println!("  addr={:08X} pc={:08X} val={:02X}", addr, pc, val);
            }
        } else {
            println!(
                "Frame {}: [50]={:08X}->{:08X}  writes=0",
                200 + f,
                v_before,
                v_after
            );
        }
    }
}
