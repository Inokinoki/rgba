use rgba::Gba;

fn main() {
    let test_dirs = [
        ("arm", "gba-tests/arm/arm.gba"),
        ("bios", "gba-tests/bios/bios.gba"),
        ("thumb", "gba-tests/thumb/thumb.gba"),
        ("memory", "gba-tests/memory/memory.gba"),
        ("unsafe", "gba-tests/unsafe/unsafe.gba"),
    ];

    for (test_name, test_path) in test_dirs.iter() {
        println!("Testing {}...", test_name);

        if let Ok(rom_data) = std::fs::read(test_path) {
            let mut gba = Gba::new();
            gba.load_rom(rom_data);

            println!("Running for {} steps...", test_name);

            let mut r12 = 0;
            for step in 0..500_000 {
                gba.step();

                r12 = gba.cpu().get_reg(12);
                if r12 != 0 && step == 499_999 {
                    println!("Test {} FAILED at step {}!", test_name, step);
                    break;
                }
            }

            let pc = gba.cpu().get_pc();
            let result = if r12 == 0 { "✅ PASS" } else { "❌ FAIL" };

            println!(
                "  {} RAN 500000 steps - Final PC: 0x{:08X} - R12: 0x{:08X} - {}",
                test_name, pc, r12, result
            );
        }
    }

    println!();
}
