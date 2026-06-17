use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..250 {
        gba.run_frame_parallel(&mut fb);
    }

    // Enable PC trace in the main loop area
    gba.mem.pc_trace_base = 0x080D2E00;
    gba.mem.pc_trace_counts = vec![0u32; 0x200]; // 0x400 bytes = 0x200 halfwords

    // Press START+A
    gba.input.press_key(KeyState::START);
    gba.input.press_key(KeyState::A);

    // Run one frame with the trace
    gba.run_frame_parallel(&mut fb);

    let state = gba.mem.read_word(0x02000074);
    let keyinput = u16::from_le_bytes([gba.mem.io()[0x130], gba.mem.io()[0x131]]);
    println!(
        "After 1 frame: State={:08X} KEYINPUT={:04X}",
        state, keyinput
    );

    // Show most-executed PCs
    let mut counts: Vec<(u32, u32)> = gba
        .mem
        .pc_trace_counts
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(i, &c)| (0x080D2E00 + (i as u32) * 2, c))
        .collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\nTop 30 most-executed PCs (080D2E00-080D3000):");
    for (pc, count) in counts.iter().take(30) {
        println!("  {:08X}: {} times", pc, count);
    }

    gba.input.release_key(KeyState::START);
    gba.input.release_key(KeyState::A);
}
