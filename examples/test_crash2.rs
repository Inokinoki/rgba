use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    // Run 3 frames
    for _ in 0..3 {
        gba.run_frame();
    }

    // Run frame 4 (first IRQ)
    gba.run_frame();
    println!("After F4: PC={:#010X}", gba.cpu().get_instruction_pc());

    // Now step through frame 5 until the second IRQ and beyond
    let mut irq_count = 0;
    let mut post_irq_steps = 0;
    for step in 0..300000 {
        let should_take = gba.mem().interrupt.should_take_interrupt();
        let mode = gba.cpu().get_mode();
        let pc = gba.cpu().get_instruction_pc();
        let thumb = gba.cpu().is_thumb_mode();

        if should_take && mode != rgba::Mode::Irq && irq_count == 0 {
            irq_count = 1;
            println!(
                "IRQ #2 entry at step {}: PC={:#010X} mode={:?} thumb={}",
                step, pc, mode, thumb
            );
        }

        gba.step();

        if irq_count == 1 {
            let new_pc = gba.cpu().get_instruction_pc();
            let new_mode = gba.cpu().get_mode();
            let new_thumb = gba.cpu().is_thumb_mode();

            if post_irq_steps < 50 || new_pc > 0x00003FFF && new_pc < 0x02000000 {
                println!(
                    "  step {}: PC={:#010X} mode={:?} thumb={}",
                    post_irq_steps, new_pc, new_mode, new_thumb
                );
            }

            post_irq_steps += 1;

            if new_pc > 0x00003FFF && new_pc < 0x02000000 {
                println!(
                    "\nCRASH at step {} ({} steps after IRQ): PC={:#010X}",
                    step, post_irq_steps, new_pc
                );
                break;
            }
        }
    }
}
