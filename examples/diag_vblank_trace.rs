use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);

    // Run 4 frames normally
    for _ in 0..4 {
        gba.run_frame();
    }

    eprintln!(
        "After 4 frames: PC=0x{:08X} halt={}",
        gba.cpu_pc(),
        gba.cpu().is_halted()
    );
    eprintln!(
        "IE=0x{:04X} IME={} IF=0x{:04X}",
        gba.mem().interrupt.ie.bits(),
        gba.mem().interrupt.ime,
        gba.mem().interrupt.if_raw.bits()
    );

    // Run frame 5 step by step, trace VBlank and IF
    let mut saw_vblank = false;
    let mut saw_hblank = false;
    let mut halt_started = false;
    let mut halt_step = 0u32;
    let mut if_changes = Vec::new();

    for step in 0..280896u32 {
        let if_before = gba.mem().interrupt.if_raw.bits();
        let halted_before = gba.cpu().is_halted();

        gba.step();

        let if_after = gba.mem().interrupt.if_raw.bits();
        let halted_after = gba.cpu().is_halted();

        if !halted_before && halted_after && !halt_started {
            halt_started = true;
            halt_step = step;
            eprintln!("Step {}: HALT entered! IF=0x{:04X}", step, if_after);
        }

        if if_before != if_after {
            if if_changes.len() < 30 {
                eprintln!(
                    "Step {}: IF changed 0x{:04X} -> 0x{:04X} halted={}",
                    step, if_before, if_after, halted_after
                );
            }
            if_changes.push((step, if_before, if_after));
        }

        // Check VBlank
        let vcount = gba.ppu().get_vcount();
        if vcount == 160 && !saw_vblank {
            saw_vblank = true;
            eprintln!(
                "Step {}: VCOUNT=160 (VBlank start) IF=0x{:04X} halted={}",
                step, if_after, halted_after
            );
        }
        if vcount == 0 && step > 1000 && !saw_hblank {
            // vcount wrapped to 0 = new frame
            saw_hblank = true;
            eprintln!(
                "Step {}: VCOUNT=0 (new frame) IF=0x{:04X} halted={}",
                step, if_after, halted_after
            );
        }
    }

    eprintln!("\nSummary:");
    eprintln!("  Halt started at step {}", halt_step);
    eprintln!("  Saw VBlank (vcount=160): {}", saw_vblank);
    eprintln!("  IF changes: {}", if_changes.len());
    eprintln!("  Final IF=0x{:04X}", gba.mem().interrupt.if_raw.bits());
    eprintln!("  Final IE=0x{:04X}", gba.mem().interrupt.ie.bits());
    eprintln!("  Final IME={}", gba.mem().interrupt.ime);
    eprintln!("  Final halt={}", gba.cpu().is_halted());

    // Check: what interrupts does the wakeup check see?
    let ie = gba.mem().interrupt.ie.bits();
    let if_ = gba.mem().interrupt.if_raw.bits();
    eprintln!(
        "  IE & IF = 0x{:04X} (wakeup = {})",
        ie & if_,
        (ie & if_) != 0
    );
}
