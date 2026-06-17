use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Track IRQ handler calls
    let mut last_irq_save = 0;
    let mut irq_calls = 0;
    let mut handler_entries = 0;

    for frame in 0..15 {
        // Run scanline by scanline
        for sl in 0..228 {
            gba.run_scanline();

            // Check if we just entered/exited IRQ
            let irq_save = gba.cpu().irq_save_count;
            if irq_save != last_irq_save {
                irq_calls += 1;
                last_irq_save = irq_save;
            }

            let pc = gba.cpu().get_pc();
            if pc >= 0x03000958 && pc < 0x03000A00 {
                handler_entries += 1;
            }
        }

        let m = gba.mem_mut();
        let vblank_ctr = m.read_word(0x03007FF8);
        let ie = m.interrupt.ie.bits();
        let if_ = m.interrupt.if_raw.bits();
        let ime = m.interrupt.ime;
        let in_int = m.interrupt.in_interrupt;
        let mode = gba.cpu().get_mode();

        println!("F{}: irq_calls={} handler_entries={} vblank_ctr={} IE=0x{:04X} IF=0x{:04X} IME={} in_int={} mode={:?}",
            frame, irq_calls, handler_entries, vblank_ctr, ie, if_, ime, in_int, mode);
    }
}
