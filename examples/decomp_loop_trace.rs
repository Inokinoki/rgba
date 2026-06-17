use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.cpu_mut().decomp_trace_enabled = true;

    for frame in 0..200 {
        for _scanline in 0..228 {
            gba.run_scanline();
        }
        if frame % 50 == 0 {
            eprintln!("Frame {}", frame);
        }
    }

    let trace = &gba.cpu().decomp_trace;
    eprintln!("Total trace entries: {}", trace.len());

    let mut in_call = false;
    let mut call_count = 0u32;
    let mut call_r1_start = 0u32;
    let mut call_r8 = 0u32;

    for (i, &(pc, opcode, regs)) in trace.iter().enumerate() {
        let in_range = (pc & !1) >= 0x080D0A40 && (pc & !1) < 0x080D0C20;

        if in_range && !in_call {
            call_count += 1;
            call_r1_start = regs[1];
            call_r8 = regs[8];
            in_call = true;
            if call_count <= 30 {
                println!("=== Call {} entry at {:08X} r0={:08X} r1={:08X} r3={:08X} r8={:08X} r9={:08X} ===",
                    call_count, pc, regs[0], regs[1], regs[3], regs[8], regs[9]);
            }
        }

        if !in_range && in_call {
            if call_count <= 30 {
                println!(
                    "=== Call {} exit r1={:08X} r8={:08X} (wrote {} bytes) ===",
                    call_count,
                    regs[1],
                    regs[8],
                    regs[1].wrapping_sub(call_r1_start)
                );
            }
            in_call = false;
        }

        if in_range && call_count <= 5 {
            if pc == 0x080D0B40 || pc == 0x080D0A96 || pc == 0x080D0A4A {
                println!(
                    "  {:4}: {:08X} cmp r1,r8  r1={:08X} r8={:08X} lt={}",
                    i,
                    pc,
                    regs[1],
                    regs[8],
                    regs[1] < regs[8]
                );
            }
            if pc == 0x080D0B42 || pc == 0x080D0A98 || pc == 0x080D0A4C {
                println!("  {:4}: {:08X} bcc taken={}", i, pc, regs[1] < regs[8]);
            }
            if pc == 0x080D0B44 || pc == 0x080D0A4E || pc == 0x080D0A9A {
                println!(
                    "  {:4}: {:08X} EXIT BL  r1={:08X} r8={:08X}",
                    i, pc, regs[1], regs[8]
                );
            }
        }
    }

    println!("\nTotal decompression calls detected: {}", call_count);
}
