use rgba::Gba;

fn main() {
    let tests = [
        ("arm", "/tmp/gba-tests/arm/arm.gba"),
        ("bios", "/tmp/gba-tests/bios/bios.gba"),
        ("thumb", "/tmp/gba-tests/thumb/thumb.gba"),
        ("memory", "/tmp/gba-tests/memory/memory.gba"),
        ("unsafe", "/tmp/gba-tests/unsafe/unsafe.gba"),
        ("nes", "/tmp/gba-tests/nes/nes.gba"),
        ("ppu-hello", "/tmp/gba-tests/ppu/hello.gba"),
        ("ppu-shades", "/tmp/gba-tests/ppu/shades.gba"),
        ("ppu-stripes", "/tmp/gba-tests/ppu/stripes.gba"),
        ("save-none", "/tmp/gba-tests/save/none.gba"),
        ("save-sram", "/tmp/gba-tests/save/sram.gba"),
        ("save-flash64", "/tmp/gba-tests/save/flash64.gba"),
        ("save-flash128", "/tmp/gba-tests/save/flash128.gba"),
    ];

    const MAX_STEPS: usize = 500_000;

    println!("═══════════════════════════════════════════════════════");
    println!("Detailed GBA Test Suite Analysis");
    println!("═══════════════════════════════════════════════════════");
    println!();

    let mut total_passed = 0;
    let mut total_failed = 0;

    for (name, path) in tests {
        print!("Test {:12} : ", name);

        let rom_data = match std::fs::read(path) {
            Ok(data) => data,
            Err(e) => {
                println!("❌ SKIP - Error reading: {}", e);
                total_failed += 1;
                continue;
            }
        };

        let mut gba = rgba::Gba::new();
        gba.load_rom(rom_data);

        // Track if we ever see R12 != 0 during execution
        let mut failed_test = None;
        let mut steps_taken = 0;

        for step in 0..MAX_STEPS {
            gba.step();

            // Check for test failure (R12 != 0)
            let r12 = gba.cpu().get_reg(12);
            if r12 != 0 && failed_test.is_none() {
                failed_test = Some((step, r12));
            }

            steps_taken = step + 1;
        }

        let r12 = gba.cpu().get_reg(12);
        let final_pc = gba.cpu().get_pc();
        let mode = format!("{:?}", gba.cpu().get_mode());

        if r12 == 0 {
            println!("✅ PASS");
            println!("               Steps: {} | PC: 0x{:08X} | Mode: {}", steps_taken, final_pc, mode);
            total_passed += 1;
        } else {
            println!("❌ FAIL");
            if let Some((step, test_num)) = failed_test {
                println!("               Test {} failed at step {}", test_num, step);
            }
            println!("               Final R12: {} | PC: 0x{:08X} | Mode: {}", r12, final_pc, mode);
            total_failed += 1;
        }
        println!();
    }

    println!("═══════════════════════════════════════════════════════");
    println!("Results: {} passed, {} failed out of {} tests", total_passed, total_failed, tests.len());
    println!("═══════════════════════════════════════════════════════");
}
