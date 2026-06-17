use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    gba.input_mut().press_key(KeyState::A);
    gba.input_mut().press_key(KeyState::START);

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Trace input struct reads for 5 frames
    gba.mem.input_reads_enabled = true;

    for f in 0..5 {
        gba.mem.input_reads.clear();
        gba.run_frame_parallel(&mut fb);
        let n = gba.mem.input_reads.len();
        if n > 0 {
            println!("Frame {}: {} input reads", 200 + f, n);
            // Group by PC
            let mut pc_counts: std::collections::HashMap<u32, Vec<u32>> =
                std::collections::HashMap::new();
            for &(addr, pc) in &gba.mem.input_reads {
                pc_counts.entry(pc).or_default().push(addr);
            }
            for (pc, addrs) in &pc_counts {
                let addrs_str: Vec<String> = addrs.iter().map(|a| format!("0x{:08X}", a)).collect();
                println!("  PC=0x{:08X}: reads from {}", pc, addrs_str.join(", "));
            }
        } else {
            println!("Frame {}: 0 input reads", 200 + f);
        }
    }
}
