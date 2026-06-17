use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run 2 frames (handler runs first at frame 2)
    for _ in 0..2 {
        gba.run_frame_parallel(&mut fb);
    }

    // Enable trace
    gba.cpu_mut().enable_trace();

    // Run scanline by scanline until IRQ fires
    for sl in 0..228 {
        let start_irq = gba.cpu().irq_save_count;
        gba.run_scanline();
        let end_irq = gba.cpu().irq_save_count;

        if end_irq != start_irq {
            println!("IRQ fired at scanline {}", sl);
            let trace = gba.cpu().get_trace();
            println!("Trace ({} entries):", trace.len());
            for (i, (pc, opcode, regs, cpsr)) in trace.iter().enumerate() {
                let mode = cpsr & 0x1F;
                let mode_str = match mode {
                    0x12 => "IRQ", 0x1F => "SYS", 0x13 => "SVC", _ => "???"
                };
                let thumb = if cpsr & 0x20 != 0 { "T" } else { "A" };
                println!("  [{}] PC=0x{:08X} op=0x{:08X} mode={}{} R0=0x{:08X} R1=0x{:08X} R2=0x{:08X} R3=0x{:08X} R12=0x{:08X} SP=0x{:08X} LR=0x{:08X} CPSR=0x{:08X}",
                    i, pc, opcode, mode_str, thumb, regs[0], regs[1], regs[2], regs[3], regs[12], regs[13], regs[14], cpsr);
            }
            break;
        }
    }
}
