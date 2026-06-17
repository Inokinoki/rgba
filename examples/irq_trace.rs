use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().irq_trace_enabled = true;

    println!("Running 300 frames with IRQ tracing...");

    for _frame in 0..300u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().irq_trace_enabled = false;

    let trace = &gba.mem().irq_trace;
    println!("\nIRQ trace entries: {}", trace.len());

    let vblank_count = trace.iter().filter(|(kind, _, _, _, _)| *kind == 0).count();
    let wake_count = trace.iter().filter(|(kind, _, _, _, _)| *kind == 1).count();
    let take_count = trace.iter().filter(|(kind, _, _, _, _)| *kind == 2).count();

    println!("VBlank fired: {} times", vblank_count);
    println!("CPU woke from halt: {} times", wake_count);
    println!("Interrupt taken: {} times", take_count);

    println!("\n=== First 20 VBlank events ===");
    for (_kind, data, ie, if_, halted) in trace.iter().filter(|(k, _, _, _, _)| *k == 0).take(20) {
        println!(
            "  VBlank: vcount={} IE={:#06X} IF={:#06X} halted={}",
            data, ie, if_, halted
        );
    }

    println!("\n=== First 20 Wake events ===");
    for (_kind, pc, ie, if_, _) in trace.iter().filter(|(k, _, _, _, _)| *k == 1).take(20) {
        println!("  Wake: PC={:#010X} IE={:#06X} IF={:#06X}", pc, ie, if_);
    }

    println!("\n=== First 20 Take IRQ events ===");
    for (_kind, pc, ie, if_, _) in trace.iter().filter(|(k, _, _, _, _)| *k == 2).take(20) {
        println!("  TakeIRQ: PC={:#010X} IE={:#06X} IF={:#06X}", pc, ie, if_);
    }

    println!("\n=== Last 10 events of each type ===");
    let vblanks: Vec<_> = trace.iter().filter(|(k, _, _, _, _)| *k == 0).collect();
    let wakes: Vec<_> = trace.iter().filter(|(k, _, _, _, _)| *k == 1).collect();
    let takes: Vec<_> = trace.iter().filter(|(k, _, _, _, _)| *k == 2).collect();

    for e in vblanks.iter().rev().take(10).rev() {
        println!(
            "  VBlank: vcount={} IE={:#06X} IF={:#06X} halted={}",
            e.1, e.2, e.3, e.4
        );
    }
    for e in wakes.iter().rev().take(10).rev() {
        println!("  Wake: PC={:#010X} IE={:#06X} IF={:#06X}", e.1, e.2, e.3);
    }
    for e in takes.iter().rev().take(10).rev() {
        println!(
            "  TakeIRQ: PC={:#010X} IE={:#06X} IF={:#06X}",
            e.1, e.2, e.3
        );
    }

    println!("\n=== Final CPU state ===");
    let pc = gba.cpu().get_instruction_pc();
    let is_thumb = gba.cpu().is_thumb_mode();
    let halted = gba.cpu().is_halted();
    let ie = gba.mem().interrupt.ie.bits();
    let if_raw = gba.mem().interrupt.if_raw.bits();
    let ime = gba.mem().interrupt.ime;
    println!(
        "PC: {:#010X} ({}) halted={}",
        pc,
        if is_thumb { "THUMB" } else { "ARM" },
        halted
    );
    println!("IE={:#06X} IF={:#06X} IME={}", ie, if_raw, ime);
}
