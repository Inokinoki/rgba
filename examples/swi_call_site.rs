use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    gba.mem_mut().swi_log_enabled = true;

    // Run to frame 2 where the first IntrWait happens
    for _ in 0..2 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now run one more frame - we'll catch registers at SWI time
    // by checking after each SWI call
    let swi_count_before = gba.mem().swi_log.len();

    // Patch: temporarily check R0/R1 in the SWI handler
    // We can't do this directly, so let's use a different approach
    // Check R0/R1 just before the SWI 0x04 call by examining the game code

    // The game calls SWI from THUMB mode. Let's find the call site.
    // From swi_trace: SWI 0x04 is call #15 (after 14 other SWIs)
    // Frame 2 PC=0x080D2F10 is after the IntrWait returns

    // Let's check what code is at the SWI call site
    // The pattern would be: MOVS R0, #0; MOVS R1, #addr; SWI 4
    // or similar

    // Actually, let's just add a trace to the SWI handler itself
    // by temporarily modifying it to print R0/R1

    // Alternative: check the call site at 0x080D0B33
    let rom = gba.mem().rom();
    // PC=0x080D0B33 is THUMB, look nearby
    let base = 0x080D0B00 - 0x08000000;
    println!("Code around 0x080D0B00:");
    for i in (0..0x80).step_by(2) {
        if base + i + 2 <= rom.len() {
            let instr = u16::from_le_bytes([rom[base + i], rom[base + i + 1]]);
            let addr = 0x080D0B00 + i;
            if instr >= 0xDF00 && instr <= 0xDFFF {
                let swi_num = instr & 0xFF;
                println!("  0x{:08X}: 0x{:04X}  SWI {}", addr, instr, swi_num);
            } else if (instr >> 8) == 0x20 || (instr >> 8) == 0x21 {
                let rd = (instr >> 8) & 7;
                let imm = instr & 0xFF;
                println!("  0x{:08X}: 0x{:04X}  MOVS R{}, #{}", addr, instr, rd, imm);
            } else if (instr >> 11) == 0x1E {
                let rd = (instr >> 8) & 7;
                let imm = instr & 0xFF;
                println!(
                    "  0x{:08X}: 0x{:04X}  MOV R{}, #0x{:02X} (shift)",
                    addr, instr, rd, imm
                );
            } else if instr == 0x4770 {
                println!("  0x{:08X}: 0x{:04X}  BX LR", addr, instr);
            } else if (instr >> 11) == 0b01001 {
                let rd = (instr >> 8) & 7;
                let off = (instr & 0xFF) * 4;
                let target = addr + 4 + off as u32;
                println!(
                    "  0x{:08X}: 0x{:04X}  LDR R{}, [PC, #{}] -> 0x{:08X}",
                    addr, instr, rd, off, target
                );
            } else {
                println!("  0x{:08X}: 0x{:04X}", addr, instr);
            }
        }
    }
}
