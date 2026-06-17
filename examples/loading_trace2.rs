use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom().to_vec();
    let mut framebuffer = vec![0u32; 240 * 160];

    let func_start = 0x080D0900u32;
    let func_end = 0x080D0CB0u32;

    gba.mem_mut().pc_trace_base = func_start;
    let trace_size = ((func_end - func_start) / 2) as usize;
    gba.mem_mut().pc_trace_counts = vec![0u32; trace_size];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().swi_log_enabled = true;

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);

        if frame % 20 == 0 {
            let total: u32 = gba.mem().pc_trace_counts.iter().sum();
            if total > 0 {
                println!(
                    "Frame {}: {} instruction executions in loading func",
                    frame, total
                );
            }
        }
    }

    gba.mem_mut().vram_log_enabled = false;
    gba.mem_mut().swi_log_enabled = false;

    println!("\n=== PC execution counts in loading function ===");
    let trace = &gba.mem().pc_trace_counts;
    let mut total_exec = 0u32;
    let mut max_count = 0u32;
    let mut max_pc = 0u32;
    for i in 0..trace.len() {
        let pc = func_start + (i as u32) * 2;
        let count = trace[i];
        total_exec += count;
        if count > max_count {
            max_count = count;
            max_pc = pc;
        }
        if count > 0 {
            let rom_off = (pc - 0x08000000) as usize;
            let opcode = if rom_off + 2 <= rom.len() {
                u16::from_le_bytes([rom[rom_off], rom[rom_off + 1]])
            } else {
                0
            };
            if count > 100 {
                println!("  {:08X}: {:04X} ({} times)", pc, opcode, count);
            }
        }
    }
    println!("Total executions in range: {}", total_exec);
    println!("Most executed: {:08X} ({} times)", max_pc, max_count);

    println!("\n=== SWI calls logged ===");
    let swi_log = &gba.mem().swi_log;
    println!("Total SWI calls: {}", swi_log.len());
    let mut swi_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &swi in swi_log {
        *swi_counts.entry(swi).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = swi_counts.iter().collect();
    sorted.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (swi, count) in sorted {
        let name = match swi {
            0x00 => "SoftReset",
            0x01 => "RegisterRamReset",
            0x02 => "Halt",
            0x03 => "Stop",
            0x04 => "IntrWait",
            0x05 => "VBlankIntrWait",
            0x06 => "Div",
            0x07 => "DivArm",
            0x08 => "DivMod",
            0x0B => "CpuSet",
            0x0C => "CpuFastSet",
            0x0D => "GetBiosChecksum",
            0x0E => "BgAffineSet",
            0x0F => "ObjAffineSet",
            0x10 => "BitUnPack",
            0x11 => "LZ77UnCompWRAM",
            0x12 => "LZ77UnCompVRAM",
            0x13 => "HuffUnComp",
            0x14 => "RLUnCompWRAM",
            0x15 => "RLUnCompVRAM",
            0x16 => "Diff8bitUnFilterWRAM",
            0x17 => "Diff8bitUnFilterVRAM",
            0x18 => "Diff16bitUnFilterWRAM",
            0x19 => "Diff16bitUnFilterVRAM",
            0x1F => "SoundBias",
            _ => "Unknown",
        };
        println!("  SWI {:#04X} ({}): {} calls", swi, name, count);
    }

    let log = &gba.mem().vram_write_log;
    let tile_writes: Vec<_> = log
        .iter()
        .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600C000)
        .collect();
    println!("\n=== Tile area writes ===");
    println!("Total: {}", tile_writes.len());
    let max_addr = tile_writes.iter().map(|(a, _, _)| *a).max().unwrap_or(0);
    println!(
        "Max addr: {:#010X} (tile {})",
        max_addr,
        (max_addr - 0x06000000) / 32
    );
}
