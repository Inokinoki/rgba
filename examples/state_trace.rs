use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..560 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.mem.timer_writes_enabled = true;

    for f in 0..15 {
        gba.mem.timer_writes.clear();
        let v50 = gba.mem.read_word(0x02000050);
        let v74 = gba.mem.read_word(0x02000074);
        gba.run_frame_parallel(&mut fb);
        let v50_after = gba.mem.read_word(0x02000050);
        let v74_after = gba.mem.read_word(0x02000074);

        let state_writes: Vec<_> = gba
            .mem
            .timer_writes
            .iter()
            .filter(|(a, _, _)| *a >= 0x02000074 && *a <= 0x02000077)
            .collect();
        let timer_writes: Vec<_> = gba
            .mem
            .timer_writes
            .iter()
            .filter(|(a, _, _)| *a >= 0x02000050 && *a <= 0x02000053)
            .collect();

        println!(
            "Frame {}: [50]={:08X}->{:08X} [74]={:08X}->{:08X}",
            561 + f,
            v50,
            v50_after,
            v74,
            v74_after
        );

        if !state_writes.is_empty() {
            println!("  STATE WRITES:");
            for &(addr, pc, val) in state_writes {
                println!("    addr={:08X} pc={:08X} val={:02X}", addr, pc, val);
            }
        }
        if !timer_writes.is_empty() {
            println!("  TIMER WRITES: {} (showing first 5)", timer_writes.len());
            for &(addr, pc, val) in timer_writes.iter().take(5) {
                println!("    addr={:08X} pc={:08X} val={:02X}", addr, pc, val);
            }
        }
    }
}
