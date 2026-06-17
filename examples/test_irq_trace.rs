use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    println!("=== BIOS IRQ dispatch test ===");

    // Check initial state
    {
        let handler = gba.mem().get_irq_handler();
        println!("Initial IRQ handler: {:#010X}", handler);

        // Check BIOS at 0x0018
        let b0 = gba.mem().bios_read_word(0x18);
        let b1 = gba.mem().bios_read_word(0x1C);
        let b2 = gba.mem().bios_read_word(0x20);
        let b3 = gba.mem().bios_read_word(0x24);
        let b4 = gba.mem().bios_read_word(0x28);
        println!("BIOS[0x18]: {:#010X}", b0);
        println!("BIOS[0x1C]: {:#010X}", b1);
        println!("BIOS[0x20]: {:#010X}", b2);
        println!("BIOS[0x24]: {:#010X}", b3);
        println!("BIOS[0x28]: {:#010X}", b4);

        // Check BIOS at 0x013C
        let stub = gba.mem().bios_read_word(0x013C);
        println!("BIOS[0x013C] (stub): {:#010X}", stub);

        // Check return stub at 0x3000
        let ret0 = gba.mem().bios_read_word(0x3000);
        let ret1 = gba.mem().bios_read_word(0x3004);
        println!("BIOS[0x3000] (return): {:#010X}", ret0);
        println!("BIOS[0x3004] (return): {:#010X}", ret1);
    }

    // Step through first 100 frames with detailed logging
    for frame in 0..50 {
        gba.run_frame();

        if frame < 10 || frame % 10 == 9 {
            let pc = gba.cpu().get_instruction_pc();
            let mode = gba.cpu().get_mode();
            let handler = gba.mem().get_irq_handler();
            let ie = gba.mem().interrupt.ie.bits();
            let ime = gba.mem().interrupt.ime;
            let if_ = gba.mem().interrupt.if_raw.bits();
            println!(
                "F{}: PC={:#010X} mode={:?} handler={:#010X} IE={:#06X} IME={} IF={:#06X}",
                frame, pc, mode, handler, ie, ime, if_
            );
        }
    }
}
