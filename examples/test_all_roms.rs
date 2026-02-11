//! Test all gba-tests ROMs

use rgba::Gba;
use std::fs;
use std::path::Path;

fn test_rom(name: &str, path: &str) -> Result<String, String> {
    let mut gba = Gba::new();

    let rom_data = match fs::read(path) {
        Ok(data) => data,
        Err(e) => return Err(format!("Failed to read ROM: {}", e)),
    };

    gba.load_rom(rom_data);

    let mut last_pc = 0;
    let mut stall_count = 0;
    const MAX_STEPS: usize = 500000;

    for i in 0..MAX_STEPS {
        let pc = gba.cpu_pc();

        // Check for stall
        if pc == last_pc {
            stall_count += 1;
            if stall_count > 100 {
                // Check if this is the idle loop (branch to self)
                let insn = gba.mem_read_word(pc);
                let is_idle_loop = insn == 0x0A000000 || // B to self (ARM)
                                     insn == 0xE7FE ||      // Infinite loop (Thumb: B . -2)
                                     insn == 0xE1A00000;   // NOP (common idle pattern)

                if is_idle_loop {
                    return Ok(format!("PASS - Reached idle loop at step {}, PC: 0x{:08X}", i, pc));
                } else {
                    return Err(format!("STALL - Stuck at PC 0x{:08X}", pc));
                }
            }
        } else {
            stall_count = 0;
            last_pc = pc;
        }

        gba.step();
    }

    Err(format!("RAN {} steps - Final PC: 0x{:08X}", MAX_STEPS, gba.cpu_pc()))
}

fn main() {
    println!("═══════════════════════════════════════════════════");
    println!("Testing gba-tests ROMs");
    println!("═══════════════════════════════════════════════════");
    println!();

    let tests = vec![
        ("arm", "/tmp/gba-tests/arm/arm.gba"),
        ("bios", "/tmp/gba-tests/bios/bios.gba"),
        ("thumb", "/tmp/gba-tests/thumb/thumb.gba"),
        ("memory", "/tmp/gba-tests/memory/memory.gba"),
        ("unsafe", "/tmp/gba-tests/unsafe/unsafe.gba"),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut results = Vec::new();

    for (name, path) in &tests {
        print!("{:10} ", name);

        match test_rom(name, path) {
            Ok(msg) => {
                println!("✅ {}", msg);
                passed += 1;
                results.push((name, true, msg));
            }
            Err(msg) => {
                println!("❌ {}", msg);
                failed += 1;
                results.push((name, false, msg));
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("Results: {} passed, {} failed out of {} tests", passed, failed, tests.len());
    println!("═══════════════════════════════════════════════════");

    if failed == 0 {
        println!();
        println!("ALL TESTS PASSED! 🎉");
    }
}
