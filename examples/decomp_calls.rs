use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.cpu_mut().decomp_trace_enabled = true;

    for frame in 0..200 {
        for _sl in 0..228 {
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
    let mut last_r1 = 0u32;
    let mut vram_tile_calls = 0u32;
    let mut vram_screen_calls = 0u32;

    for (i, &(pc, _opcode, regs)) in trace.iter().enumerate() {
        let in_range = (pc & !1) >= 0x080D0A40 && (pc & !1) < 0x080D0C20;

        if in_range && !in_call {
            call_count += 1;
            call_r1_start = regs[1];
            call_r8 = regs[8];
            in_call = true;
            last_r1 = regs[1];
        }

        if in_range && in_call {
            last_r1 = regs[1];
        }

        if !in_range && in_call {
            let r1_in_vram_tile = call_r1_start >= 0x06000000 && call_r1_start < 0x0600C000;
            let r1_in_vram_screen = call_r1_start >= 0x0600C000 && call_r1_start < 0x06010000;
            let r8_in_vram_tile = call_r8 >= 0x06000000 && call_r8 < 0x0600C000;
            let r8_in_vram_screen = call_r8 >= 0x0600C000 && call_r8 < 0x06010000;
            let r1_in_vram = call_r1_start >= 0x06000000 && call_r1_start < 0x06010000;
            let r8_in_vram = call_r8 >= 0x06000000 && call_r8 < 0x06010000;

            let is_vram = r1_in_vram || r8_in_vram;

            if is_vram {
                let wrote = last_r1.wrapping_sub(call_r1_start);
                let r1_type = if r1_in_vram_tile {
                    "TILE"
                } else if r1_in_vram_screen {
                    "SCR"
                } else {
                    "V-other"
                };
                let r8_type = if r8_in_vram_tile {
                    "TILE"
                } else if r8_in_vram_screen {
                    "SCR"
                } else {
                    "V-other"
                };
                println!("Call {}: r1={:08X}({}) r8={:08X}({}) last_r1={:08X} wrote={} r0={:08X} r3={:08X} r9={:08X}",
                    call_count, call_r1_start, r1_type, call_r8, r8_type, last_r1, wrote,
                    regs[0], regs[3], regs[9]);
                if r1_in_vram_tile || r8_in_vram_tile {
                    vram_tile_calls += 1;
                }
                if r1_in_vram_screen || r8_in_vram_screen {
                    vram_screen_calls += 1;
                }
            }
            in_call = false;
        }
    }

    if in_call {
        println!("Call {} still in progress at end", call_count);
    }

    println!(
        "\nTotal calls: {}  VRAM-tile: {}  VRAM-screen: {}",
        call_count, vram_tile_calls, vram_screen_calls
    );
}
