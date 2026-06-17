use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..8 { gba.run_frame_parallel(&mut fb); }

    // Trace frame 8 with full coverage
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x100000]; // 0x08000000-0x08200000

    gba.run_frame_parallel(&mut fb);
    
    let trace = &gba.mem().pc_trace_counts;
    
    // Print ALL PCs with >0 hits, in order
    println!("=== All PCs at frame 8 (in address order) ===");
    for i in 0..trace.len() {
        if trace[i] > 0 {
            let pc = 0x08000000 + (i as u32) * 2;
            println!("0x{:08X}: {}", pc, trace[i]);
        }
    }
}
