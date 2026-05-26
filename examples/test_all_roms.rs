use rgba::Gba;
use std::fs;

fn test_rom(path: &str) -> Result<String, String> {
    let mut gba = Gba::new();

    let rom_data = match fs::read(path) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to read ROM: {}", e)),
    };

    gba.load_rom(rom_data);

    const MAX_STEPS: usize = 500_000;

    for _ in 0..MAX_STEPS {
        gba.step();
    }

    let r12 = gba.cpu().get_reg(12);
    let pc = gba.cpu().get_pc();

    if r12 == 0 {
        Ok(format!(
            "PASS - R12=0 after {} steps, PC=0x{:08X}",
            MAX_STEPS, pc
        ))
    } else {
        Err(format!(
            "FAIL - R12={} after {} steps, PC=0x{:08X}",
            r12, MAX_STEPS, pc
        ))
    }
}

fn main() {
    println!("═══════════════════════════════════════════════════");
    println!("Testing gba-tests ROMs");
    println!("═══════════════════════════════════════════════════");
    println!();

    let tests = vec![
        ("arm", "gba-tests/arm/arm.gba"),
        ("bios", "gba-tests/bios/bios.gba"),
        ("thumb", "gba-tests/thumb/thumb.gba"),
        ("memory", "gba-tests/memory/memory.gba"),
        ("unsafe", "gba-tests/unsafe/unsafe.gba"),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (name, path) in &tests {
        print!("{:10} ", name);

        match test_rom(path) {
            Ok(msg) => {
                println!("✅ {}", msg);
                passed += 1;
            }
            Err(msg) => {
                println!("❌ {}", msg);
                failed += 1;
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════");
    println!(
        "Results: {} passed, {} failed out of {} tests",
        passed,
        failed,
        tests.len()
    );
    println!("═══════════════════════════════════════════════════");

    if failed == 0 {
        println!();
        println!("ALL TESTS PASSED!");
    }
}
