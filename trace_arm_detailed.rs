use rgba::Gba;

fn main() {
    let rom_data = std::fs::read("/tmp/gba-tests/arm/arm.gba").unwrap();
    let mut gba = Gba::new();
    gba.load_rom(rom_data);

    println!("Tracing ARM execution looking for non-word-aligned PC...\n");

    for i in 1..50 {
        let pc_before = gba.cpu().get_pc();
        let thumb_before = gba.cpu().is_thumb_mode();
        
        gba.step();
        
        let pc_after = gba.cpu().get_pc();
        let thumb_after = gba.cpu().is_thumb_mode();

        // Check for non-word-aligned PC in ARM mode
        if !thumb_after && (pc_after & 3 != 0) {
            println!("STEP {}: PC became non-word-aligned!", i);
            println!("  Before: 0x{:08X} ({})", pc_before, if thumb_before { "Thumb" } else { "ARM" });
            println!("  After:  0x{:08X} ({})", pc_after, if thumb_after { "Thumb" } else { "ARM" });
            
            // The instruction that was executed is at pc_before - 8 (due to pipeline)
            let pc_exec = pc_before.wrapping_sub(8);
            let rom_offset = pc_exec.wrapping_sub(0x08000000);
            if rom_offset < 8824 {
                let insn = gba.mem_mut().read_word(pc_exec);
                println!("  Instruction at 0x{:08X}: 0x{:08X}", pc_exec, insn);
            }
            break;
        }
        
        // Also check for large jumps
        let diff = if pc_after >= pc_before {
            pc_after - pc_before
        } else {
            pc_before.wrapping_sub(pc_after)
        };
        
        if diff > 0x1000 {
            println!("STEP {}: Large jump", i);
            println!("  0x{:08X} -> 0x{:08X}", pc_before, pc_after);
            
            let pc_exec = pc_before.wrapping_sub(8);
            let rom_offset = pc_exec.wrapping_sub(0x08000000);
            if rom_offset < 8824 {
                let insn = gba.mem_mut().read_word(pc_exec);
                println!("  Instruction at 0x{:08X}: 0x{:08X}", pc_exec, insn);
                
                // Check if BLX
                if (insn & 0xFE000000) == 0xFA000000 {
                    println!("  -> BLX (H=0)");
                } else if (insn & 0xFF000000) == 0xFB000000 {
                    println!("  -> BLX (H=1)");
                }
            }
        }
    }
}
