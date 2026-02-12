use rgba::Gba;

fn main() {
    let test_dirs = [
        ("arm", "/tmp/gba-tests/arm/arm.gba"),
        ("bios", "/tmp/gba-tests/bios/bios.gba"),
        ("thumb", "/tmp/gba-tests/thumb/thumb.gba"),
        ("memory", "/tmp/gba-tests/memory/memory.gba"),
        ("unsafe", "/tmp/gba-tests/unsafe/unsafe.gba"),
    ];

    for (test_name, test_path) in test_dirs.iter() {
        println!("Testing {}...", test_name);

        if let Ok(rom_data) = std::fs::read(test_path) {
            let mut gba = rgba::Gba::new();
            gba.load_rom(rom_data);

            println!("Running for {} steps...", test_name);

            for _ in 0..500_000 {
                gba.step();

                let r12 = gba.cpu().get_reg(12);
            if r12 != 0 && _ == 500_000 - 1 {
                    println!("Test {} FAILED at step {}!", test_name);
                    break;
                }
            }

            let pc = gba.cpu().get_pc();
            println!("Final PC: 0x{:08X}", pc);

            let result = if r12 == 0 { "✅ PASS" } else { "❌ FAIL" };

            println!("  {} RAN {} steps - Final PC: 0x{:08X} - R12: 0x{:08X} - {}",
                test_name, result, pc);
        }
    }

    println!();
}
}
