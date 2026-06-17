use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    for _ in 0..5 {
        gba.run_frame();
    }

    // Run one more frame, find the VBlank wakeup and trace interrupt handling
    let mut found_wakeup = false;

    for step in 0..280896u32 {
        let halted = gba.cpu().is_halted();
        let if_val = gba.mem().interrupt.if_raw.bits();
        let ie_val = gba.mem().interrupt.ie.bits();
        let ime = gba.mem().interrupt.ime;
        let in_int = gba.mem().interrupt.in_interrupt;
        let mode = gba.cpu_get_cpsr() & 0x1F;
        let pc = gba.cpu_pc();

        // Right before wakeup (when halted and VBlank just fired)
        if halted && (if_val & ie_val) != 0 && !found_wakeup {
            found_wakeup = true;
            eprintln!("Step {}: About to wakeup!", step);
            eprintln!(
                "  IF=0x{:04X} IE=0x{:04X} IE&IF=0x{:04X}",
                if_val,
                ie_val,
                if_val & ie_val
            );
            eprintln!("  IME={} in_interrupt={}", ime, in_int);
            eprintln!("  should_wake={}", (if_val & ie_val) != 0);
            eprintln!(
                "  should_take_int={}",
                gba.mem().interrupt.should_take_interrupt()
            );
            eprintln!("  get_pending={:?}", gba.mem().interrupt.get_pending());
            eprintln!(
                "  CPSR=0x{:08X} mode={} IRQ_disable={}",
                gba.cpu_get_cpsr(),
                mode,
                (gba.cpu_get_cpsr() >> 7) & 1
            );
            eprintln!("  PC=0x{:08X}", pc);
        }

        gba.step();

        if found_wakeup && step < 300000 {
            let new_halted = gba.cpu().is_halted();
            let new_mode = gba.cpu_get_cpsr() & 0x1F;
            let new_pc = gba.cpu_pc();
            if !new_halted || new_mode != mode || new_pc != pc {
                eprintln!(
                    "Step {}: After step: halted={} mode={} PC=0x{:08X} IF=0x{:04X}",
                    step + 1,
                    new_halted,
                    new_mode,
                    new_pc,
                    gba.mem().interrupt.if_raw.bits()
                );
                if step > 240250 {
                    break;
                }
            }
        }
    }
}
