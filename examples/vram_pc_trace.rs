use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..7u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().vram_log_enabled = false;

    let log = &gba.mem().vram_write_log;
    println!("VRAM write log entries: {}", log.len());

    let mut unique_pcs: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for (addr, pc, _val) in log.iter() {
        if *addr >= 0x06000000 && *addr < 0x0600C000 {
            unique_pcs.insert(*pc);
        }
    }

    println!(
        "Unique PCs writing to VRAM tile area (0x06000000-0x0600BFFF): {}",
        unique_pcs.len()
    );
    let mut sorted_pcs: Vec<u32> = unique_pcs.into_iter().collect();
    sorted_pcs.sort();
    for pc in sorted_pcs.iter().take(20) {
        println!("  PC: {:#010X}", pc);
    }

    let rom = gba.mem().rom().to_vec();

    println!("\n=== Disassembly around loading PCs ===");
    for &pc_val in sorted_pcs.iter().take(5) {
        if pc_val >= 0x08000000 {
            let rom_off = (pc_val - 0x08000000) as usize & !1;
            let is_thumb = pc_val & 1 != 0;
            println!(
                "\nCode at {:#010X} ({}):",
                pc_val & !1,
                if is_thumb { "THUMB" } else { "ARM" }
            );
            let base = rom_off.saturating_sub(10);
            for i in 0..20u32 {
                let off = base + i as usize * 2;
                if off + 2 <= rom.len() {
                    let opcode = u16::from_le_bytes([rom[off], rom[off + 1]]);
                    let addr = 0x08000000 + off as u32;
                    let marker = if addr == (pc_val & !1) { ">>>" } else { "   " };
                    println!("{}{:08X}: {:04X}", marker, addr, opcode);
                }
            }
        }
    }

    println!("\n=== VRAM write sequence (first 50 tile writes) ===");
    let mut count = 0;
    for (addr, pc, val) in log.iter() {
        if *addr >= 0x06000000 && *addr < 0x0600C000 {
            println!(
                "  [{:04}] addr={:#010X} pc={:#010X} val={:02X}",
                count, addr, pc, val
            );
            count += 1;
            if count >= 50 {
                break;
            }
        }
    }

    println!("\n=== Last 20 tile writes ===");
    let tile_writes: Vec<_> = log
        .iter()
        .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600C000)
        .collect();
    for (addr, pc, val) in tile_writes.iter().rev().take(20).rev() {
        println!("  addr={:#010X} pc={:#010X} val={:02X}", addr, pc, val);
    }
}
