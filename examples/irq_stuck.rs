use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    // Run to frame 5
    for _ in 0..5 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Enable IRQ trace for frames 5-10
    gba.mem_mut().irq_trace_enabled = true;

    for frame in 5..12 {
        gba.mem_mut().irq_trace.clear();
        gba.run_frame_parallel(&mut framebuffer);

        let trace = &gba.mem().irq_trace;
        println!("\n=== Frame {} : {} IRQ events ===", frame, trace.len());
        for (i, &(ty, pc_or_sl, ie, irf, halted)) in trace.iter().enumerate() {
            let ty_str = match ty {
                0 => "VBLANK_REQ",
                1 => "HALT_WAKE",
                2 => "IRQ_TAKEN",
                _ => "???",
            };
            if ty == 0 {
                println!("  [{}] {} scanline={} IE={:#06X} IF={:#06X} halted={}", i, ty_str, pc_or_sl, ie, irf, halted);
            } else {
                println!("  [{}] {} PC={:#010X} IE={:#06X} IF={:#06X} halted={}", i, ty_str, pc_or_sl, ie, irf, halted);
            }
        }
    }
}
