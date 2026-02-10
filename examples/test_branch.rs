//! Test ARM branch calculation directly

use rgba::Gba;

fn main() {
    let mut gba = Gba::new();

    // Load the BIOS test ROM
    match gba.load_rom_path("/home/ubuntu/Builds/gba-tests/bios/bios.gba") {
        Ok(_) => println!("Loaded ROM"),
        Err(e) => {
            eprintln!("Failed to load ROM: {}", e);
            return;
        }
    }

    println!("\nFirst word at 0x08000000:");
    let word = gba.mem().read_word(0x08000000);
    println!("  0x{:08X}", word);

    println!("\nExpected: 0xEA00002E (branch to 0x080000C0)");
    println!("Got:      0x{:08X}", word);

    if word == 0xEA00002E {
        println!("\n✓ Instruction matches!");

        // Calculate target
        let offset_imm = (word & 0x00FFFFFF) as i32;
        let offset = if (offset_imm & 0x800000) != 0 {
            (((offset_imm as u32) | 0xFF000000) as i32) << 2
        } else {
            offset_imm << 2
        };

        println!("\nOffset calculation:");
        println!("  offset_imm = {}", offset_imm);
        println!("  offset = {}", offset);
        println!("  target = 0x08000000 + 8 + {} = 0x{:08X}", offset, 0x08000000 + 8 + offset as u32);

        // Now execute one step
        println!("\nExecuting one step...");
        let pc_before = gba.cpu().get_pc();
        gba.step();
        let pc_after = gba.cpu().get_pc();

        println!("  PC before: 0x{:08X}", pc_before);
        println!("  PC after:  0x{:08X}", pc_after);

        if pc_after == 0x080000C0 {
            println!("\n✓ Branch worked correctly!");
        } else {
            println!("\n✗ Branch FAILED!");
            println!("  Expected: 0x080000C0");
            println!("  Got:      0x{:08X}", pc_after);

            // Check what's at 0x080000C0
            let target_insn = gba.mem().read_word(0x080000C0);
            println!("\nInstruction at target (0x080000C0): 0x{:08X}", target_insn);
        }
    } else {
        println!("\n✗ Instruction mismatch!");
    }
}
