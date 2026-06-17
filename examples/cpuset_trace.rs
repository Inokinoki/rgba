use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    gba.mem_mut().cpu_set_log_enabled = true;
    gba.mem_mut().swi_log_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let cpu_set_log = &gba.mem().cpu_set_log;
    println!("=== CpuSet Calls ===");
    for (i, &(src, dst, cnt)) in cpu_set_log.iter().enumerate() {
        let fill = (cnt >> 24) & 1;
        let is_32 = (cnt >> 26) & 1;
        let count = cnt & 0x1FFFFF;
        println!(
            "CpuSet {}: src={:#010X} dst={:#010X} cnt={:#010X} fill={} is32={} count={}",
            i, src, dst, cnt, fill, is_32, count
        );

        let mem = gba.mem();
        if (dst & 0x06000000) == 0x06000000 {
            println!("  -> Writes to VRAM at offset {:#X}", dst - 0x06000000);
        }
        if (src & 0x08000000) == 0x08000000 {
            let rom_offset = src - 0x08000000;
            println!("  -> Reads from ROM at offset {:#X}", rom_offset);
            let rom = mem.rom();
            if rom_offset as usize + 16 <= rom.len() {
                print!("  ROM data: ");
                for j in 0..16 {
                    print!("{:02X} ", rom[rom_offset as usize + j]);
                }
                println!();
            }
        }
    }

    let swi_log = &gba.mem().swi_log;
    let mut swi_seq = Vec::new();
    let mut last_swi = 0xFF;
    for &swi in swi_log.iter() {
        if swi != last_swi {
            swi_seq.push((swi, 1u32));
            last_swi = swi;
        } else if let Some(last) = swi_seq.last_mut() {
            last.1 += 1;
        }
    }
    println!("\n=== SWI Call Sequence ===");
    for (swi, count) in &swi_seq {
        let name = match swi {
            0x01 => "RegisterRamReset",
            0x02 => "Halt",
            0x04 => "IntrWait",
            0x05 => "VBlankIntrWait",
            0x06 => "Div",
            0x0B => "CpuSet",
            0x0C => "CpuFastSet",
            _ => "Unknown",
        };
        print!("SWI_{:#04X}({})x{} ", swi, name, count);
    }
    println!();
}
