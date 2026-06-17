use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    // Run 3 frames to get to halt loop
    for _ in 0..3 {
        gba.run_frame();
    }
    println!(
        "After 3 frames: PC={:#010X}",
        gba.cpu().get_instruction_pc()
    );

    let mut last_irq_pc = 0u32;
    let mut irq_count = 0;
    let mut steps = 0u64;
    let frame_cycles = 280896u64;

    // Run frames 4 through the crash
    for frame in 4..10 {
        for _ in 0..280896 {
            let should_take = gba.mem().interrupt.should_take_interrupt();
            let mode = gba.cpu().get_mode();
            let pc = gba.cpu().get_instruction_pc();

            if should_take && mode != rgba::Mode::Irq {
                irq_count += 1;
                last_irq_pc = pc;
                if irq_count <= 5 || irq_count % 100 == 0 {
                    println!("IRQ #{} at PC={:#010X} (step {})", irq_count, pc, steps);
                }
            }

            gba.step();
            steps += 1;

            if pc > 0x00003FFF && pc < 0x02000000 {
                println!(
                    "\nCRASH at frame {} step {} (IRQ count={}): PC={:#010X}",
                    frame, steps, irq_count, pc
                );
                println!("Last IRQ entry was at PC={:#010X}", last_irq_pc);
                return;
            }
        }
        println!(
            "Frame {} done: PC={:#010X} (IRQs so far: {})",
            frame,
            gba.cpu().get_instruction_pc(),
            irq_count
        );
    }
}
