//! Run all GBA tests from gba-tests repository

use rgba::Gba;
use std::fs;

fn test_rom(path: &str, steps: usize, idle_addrs: &[u32]) -> Result<String, String> {
    let rom_data = fs::read(path).expect("Failed to read ROM");
    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    gba.cpu_mut().set_pc(0x08000000);

    let name = std::path::Path::new(path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    // Run until idle loop or step limit
    for _ in 0..steps {
        let pc = gba.cpu().get_pc();

        // Check if at idle loop
        for &idle_addr in idle_addrs {
            if pc == idle_addr {
                let r12 = gba.cpu().get_reg(12);
                if r12 == 0 {
                    return Ok(format!("PASS - reached idle loop at 0x{:08X}", idle_addr));
                } else {
                    return Err(format!("FAIL - R12={} at idle loop", r12));
                }
            }
        }

        gba.step();
    }

    // Check if test passed
    let r12 = gba.cpu().get_reg(12);
    let pc = gba.cpu().get_pc();

    if r12 == 0 {
        Ok(format!("PASS - R12=0 after {} steps", steps))
    } else {
        Err(format!(
            "FAIL - R12={} after {} steps, PC=0x{:08X}",
            r12, steps, pc
        ))
    }
}

fn main() {
    println!("=== Running All GBA Tests ===\n");

    let tests = vec![
        // PPU Tests
        ("gba-tests/ppu/shades.gba", 100000, vec![0x0800015C]),
        ("gba-tests/ppu/stripes.gba", 100000, vec![0x08000140]),
        ("gba-tests/ppu/hello.gba", 500000, vec![0x08000160]),
        // CPU Tests
        ("gba-tests/arm/arm.gba", 500000, vec![]),
        ("gba-tests/thumb/thumb.gba", 500000, vec![]),
        // Memory Tests
        ("gba-tests/memory/memory.gba", 500000, vec![]),
        ("gba-tests/unsafe/unsafe.gba", 500000, vec![]),
        // BIOS Test
        ("gba-tests/bios/bios.gba", 500000, vec![]),
        // Save Tests
        ("gba-tests/save/none.gba", 500000, vec![]),
        ("gba-tests/save/sram.gba", 500000, vec![]),
        ("gba-tests/save/flash64.gba", 500000, vec![]),
        ("gba-tests/save/flash128.gba", 500000, vec![]),
        // NES Test
        ("gba-tests/nes/nes.gba", 500000, vec![]),
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (path, steps, idle_addrs) in tests {
        print!("Testing {}... ", path);
        match test_rom(path, steps, &idle_addrs) {
            Ok(msg) => {
                println!("✓ {}", msg);
                passed += 1;
            }
            Err(msg) => {
                println!("✗ {}", msg);
                failed += 1;
            }
        }
    }

    println!();
    println!("=== Results: {} passed, {} failed ===", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
