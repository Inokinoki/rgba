use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    for _ in 0..3 {
        gba.run_frame();
    }

    let handler = gba.mem().get_irq_handler();
    println!(
        "Handler pointer: {:#010X} (bit0={} mode={})",
        handler,
        handler & 1,
        if handler & 1 != 0 { "Thumb" } else { "ARM" }
    );

    // Read code at handler using mem_read_word
    let addr = handler & !1u32;
    println!("Code at {:#010X}:", addr);
    for i in 0..8 {
        let w = gba.mem_read_word(addr + i * 4);
        println!("  [{:#010X}] = {:#010X}", addr + i * 4, w);
    }

    // Step until we see the crash
    println!("\n=== Tracing to crash ===");
    let mut saw_irq = false;
    for step in 0..500000 {
        let should_take = gba.mem().interrupt.should_take_interrupt();
        let mode = gba.cpu().get_mode();
        let pc = gba.cpu().get_instruction_pc();
        let thumb = gba.cpu().is_thumb_mode();

        if should_take && mode != rgba::Mode::Irq && !saw_irq {
            println!(
                "Step {}: IRQ pending! PC={:#010X} mode={:?} thumb={}",
                step, pc, mode, thumb
            );
            saw_irq = true;
        }

        gba.step();

        if saw_irq {
            let new_pc = gba.cpu().get_instruction_pc();
            let new_mode = gba.cpu().get_mode();
            let new_thumb = gba.cpu().is_thumb_mode();
            println!(
                "  -> PC={:#010X} mode={:?} thumb={}",
                new_pc, new_mode, new_thumb
            );

            if new_pc > 0x00003FFF && new_pc < 0x02000000 {
                println!("\nCRASH: PC in unmapped memory at {:#010X}", new_pc);
                break;
            }

            if new_mode != rgba::Mode::Irq && saw_irq {
                // Left IRQ mode, count steps
            }
        }

        if pc > 0x00003FFF && pc < 0x02000000 {
            println!("Step {}: PC in invalid range {:#010X}", step, pc);
            break;
        }
    }
}
