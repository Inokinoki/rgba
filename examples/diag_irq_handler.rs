use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    for _ in 0..5 {
        gba.run_frame();
    }

    // Run one more frame step by step, capture VBlank handler execution
    let mut in_handler = false;
    let mut handler_pcs: Vec<(u32, u32, bool)> = Vec::new(); // (step, pc, halted)
    let mut vblank_fired = false;
    let mut vblank_step = 0u32;

    for step in 0..280896u32 {
        let halted_before = gba.cpu().is_halted();
        let pc_before = gba.cpu_pc();
        let mode = gba.cpu_get_cpsr() & 0x1F;

        gba.step();

        let halted_after = gba.cpu().is_halted();
        let pc_after = gba.cpu_pc();
        let mode_after = gba.cpu_get_cpsr() & 0x1F;

        // Detect VBlank
        let if_val = gba.mem().interrupt.if_raw.bits();
        if (if_val & 1) != 0 && !vblank_fired {
            vblank_fired = true;
            vblank_step = step;
            eprintln!(
                "Step {}: VBlank IF fired! PC=0x{:08X} mode={} halted={}",
                step, pc_after, mode_after, halted_after
            );
        }

        // Track when CPU transitions from halted to not-halted
        if halted_before && !halted_after {
            in_handler = true;
            eprintln!(
                "Step {}: WAKEUP! PC=0x{:08X} mode={} (was halted)",
                step, pc_after, mode_after
            );
        }

        // Track CPU execution while in handler
        if in_handler && !halted_after {
            let instr_pc = gba.cpu().get_instruction_pc();
            if handler_pcs.len() < 100 {
                handler_pcs.push((step, instr_pc, mode_after == 0x12)); // 0x12 = IRQ mode
            }
        }

        // Track when CPU halts again
        if !halted_before && halted_after && in_handler {
            eprintln!(
                "Step {}: RE-HALT! PC=0x{:08X} mode={}",
                step, pc_after, mode_after
            );
            eprintln!("  Handler ran for {} steps", step - vblank_step);
            in_handler = false;
        }
    }

    eprintln!("\nHandler PCs (first {}):", handler_pcs.len());
    for (step, pc, is_irq) in &handler_pcs {
        let mode_str = if *is_irq { "IRQ" } else { "SYS" };
        eprintln!("  Step {}: PC=0x{:08X} ({})", step, pc, mode_str);
    }

    // Check CPSR mode bits
    eprintln!("\nMode values: 0x10=USR 0x1F=SYS 0x12=IRQ 0x13=FIQ");
}
