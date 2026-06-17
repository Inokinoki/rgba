use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);

    // Run frame by frame, trace IE/IME changes
    for frame in 0..6u32 {
        let io = gba.mem().io();
        let ie_before = u16::from_le_bytes([io[0x200], io[0x201]]);
        let ime_before = u16::from_le_bytes([io[0x208], io[0x209]]);
        let dispcnt_before = u16::from_le_bytes([io[0], io[1]]);
        let pc_before = gba.cpu_pc();
        eprintln!("\n=== Starting frame {} ===", frame);
        eprintln!(
            "  Before: PC=0x{:08X} DC=0x{:04X} IE=0x{:04X} IME={} halt={}",
            pc_before,
            dispcnt_before,
            ie_before,
            ime_before,
            gba.cpu().is_halted()
        );

        gba.run_frame();

        let io = gba.mem().io();
        let ie_after = u16::from_le_bytes([io[0x200], io[0x201]]);
        let ime_after = u16::from_le_bytes([io[0x208], io[0x209]]);
        let dispcnt_after = u16::from_le_bytes([io[0], io[1]]);
        let pc_after = gba.cpu_pc();
        eprintln!(
            "  After:  PC=0x{:08X} DC=0x{:04X} IE=0x{:04X} IME={} halt={}",
            pc_after,
            dispcnt_after,
            ie_after,
            ime_after,
            gba.cpu().is_halted()
        );
    }

    // Now run step-by-step through frame 4 to find where halt happens
    eprintln!("\n=== Step-by-step through frame 4 ===");
    let mut gba2 = Gba::new();
    gba2.load_rom(std::fs::read(rom_path).unwrap());
    for _ in 0..4 {
        gba2.run_frame();
    }

    // Check state before frame 5
    {
        let io = gba2.mem().io();
        let ie = u16::from_le_bytes([io[0x200], io[0x201]]);
        let ime = u16::from_le_bytes([io[0x208], io[0x209]]);
        let dc = u16::from_le_bytes([io[0], io[1]]);
        eprintln!(
            "Before frame 5: PC=0x{:08X} DC=0x{:04X} IE=0x{:04X} IME={} halt={}",
            gba2.cpu_pc(),
            dc,
            ie,
            ime,
            gba2.cpu().is_halted()
        );
    }

    // Run steps and trace IE/IME/PC changes until halt
    let mut last_ie = 0u16;
    let mut last_ime = 0u16;
    let mut last_pc = 0u32;
    let mut halted = false;
    let mut change_count = 0;
    for step in 0..300000u32 {
        gba2.step();
        let io = gba2.mem().io();
        let ie = u16::from_le_bytes([io[0x200], io[0x201]]);
        let ime = u16::from_le_bytes([io[0x208], io[0x209]]);
        let pc = gba2.cpu_pc();
        let h = gba2.cpu().is_halted();

        if ie != last_ie || ime != last_ime || (h && !halted) || change_count < 20 {
            if change_count < 50 {
                eprintln!(
                    "  Step {}: PC=0x{:08X} IE=0x{:04X} IME={} halt={}",
                    step, pc, ie, ime, h
                );
            }
            change_count += 1;
        }

        if h && !halted {
            halted = true;
            // Check what instruction the CPU is about to execute
            eprintln!(
                "  >>> HALT entered at step {}! PC=0x{:08X} IE=0x{:04X} IME={}",
                step, pc, ie, ime
            );
            // Check IF
            let if_ = u16::from_le_bytes([io[0x202], io[0x203]]);
            eprintln!("  >>> IF=0x{:04X} (IF & IE = 0x{:04X})", if_, if_ & ie);
            // Check DISPCNT
            let dc = u16::from_le_bytes([io[0], io[1]]);
            eprintln!(
                "  >>> DC=0x{:04X} DISPSTAT=0x{:04X} VCOUNT={}",
                dc,
                u16::from_le_bytes([io[4], io[5]]),
                u16::from_le_bytes([io[6], io[7]])
            );
            break;
        }

        last_ie = ie;
        last_ime = ime;
        last_pc = pc;
    }

    // Also trace backward: check what the CPU does right before halt
    // Re-run and trace the last N instructions before halt
    eprintln!("\n=== Tracing last instructions before halt ===");
    let mut gba3 = Gba::new();
    gba3.load_rom(std::fs::read(rom_path).unwrap());
    for _ in 0..4 {
        gba3.run_frame();
    }

    // Collect last 20 instruction PCs before halt
    let mut recent_pcs: Vec<(u32, u32)> = Vec::new(); // (step, pc)
    for step in 0..300000u32 {
        let pc = gba3.cpu_pc();
        let h = gba3.cpu().is_halted();
        if h {
            eprintln!("  Halt at step {}! Last 20 PCs:", step);
            for (s, p) in recent_pcs.iter().rev().take(20).rev() {
                eprintln!("    Step {}: PC=0x{:08X}", s, p);
            }
            break;
        }
        gba3.step();
        recent_pcs.push((step, pc));
        if recent_pcs.len() > 100 {
            recent_pcs.remove(0);
        }
    }
}
