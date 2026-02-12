use rgba::Gba;

fn main() {
    let rom_path = "/tmp/gba-tests/arm/arm.gba";

    println!("Tracing ARM test execution...");
    println!("═══════════════════════════════════════════════════════");

    let rom_data = match std::fs::read(rom_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading {}: {}", rom_path, e);
            std::process::exit(1);
        }
    };

    let mut gba = rgba::Gba::new();
    gba.load_rom(rom_data);

    let mut prev_pc = 0;
    let mut pc_changes = 0;

    // Trace first 1000 instructions
    for step in 0..1000 {
        let pc = gba.cpu().get_pc();
        let r12 = gba.cpu().get_reg(12);
        let r15 = gba.cpu().get_reg(15);
        let mode = gba.cpu().get_mode();
        let n = gba.cpu().get_flag_n();
        let z = gba.cpu().get_flag_z();
        let c = gba.cpu().get_flag_c();
        let v = gba.cpu().get_flag_v();

        if step < 50 || pc != prev_pc {
            println!("Step {:4}: PC=0x{:08X} R15=0x{:08X} R12=0x{:08X} NZCV={:04b} Mode={:?}",
                step, pc, r15, r12,
                (n as u8) << 3 | (z as u8) << 2 | (c as u8) << 1 | v as u8,
                mode);
            prev_pc = pc;
            pc_changes += 1;
        }

        if r12 != 0 {
            println!("\n❌ Test failed at step {}! R12 = {}", step, r12);
            break;
        }

        gba.step();
    }

    println!("\n═══════════════════════════════════════════════════════");
    println!("Traced {} instructions with {} PC changes", 1000, pc_changes);
}
