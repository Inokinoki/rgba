use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let rom = gba.mem().rom().to_vec();

    println!("=== Full loading function disassembly (0x080D0A00-0x080D0C00) ===");
    for i in 0..256u32 {
        let off = 0x0A00 + (i as usize) * 2;
        if off + 2 <= rom.len() {
            let opcode = u16::from_le_bytes([rom[off], rom[off + 1]]);
            let addr = 0x080D0000 + off as u32;
            let marker = if addr == 0x080D0B74 || addr == 0x080D0BEB || addr == 0x080D0BFA {
                ">>>"
            } else {
                "   "
            };
            println!("{}{:08X}: {:04X}", marker, addr, opcode);
        }
    }

    println!("\n=== Extended context (0x080D0900-0x080D0A00) ===");
    for i in 0..128u32 {
        let off = 0x0900 + (i as usize) * 2;
        if off + 2 <= rom.len() {
            let opcode = u16::from_le_bytes([rom[off], rom[off + 1]]);
            let addr = 0x080D0000 + off as u32;
            println!("   {:08X}: {:04X}", addr, opcode);
        }
    }

    println!("\n=== Extended context (0x080D0C00-0x080D0D00) ===");
    for i in 0..128u32 {
        let off = 0x0C00 + (i as usize) * 2;
        if off + 2 <= rom.len() {
            let opcode = u16::from_le_bytes([rom[off], rom[off + 1]]);
            let addr = 0x080D0000 + off as u32;
            println!("   {:08X}: {:04X}", addr, opcode);
        }
    }

    let mut framebuffer = vec![0u32; 240 * 160];
    gba.mem_mut().vram_log_enabled = true;

    for frame in 0..200u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().vram_log_enabled = false;

    let log = &gba.mem().vram_write_log;
    let mut pcs: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for (addr, pc, _val) in log.iter() {
        if *addr >= 0x06000000 && *addr < 0x0600C000 {
            *pcs.entry(*pc).or_insert(0) += 1;
        }
    }
    println!("\n=== VRAM write PCs (tile area) ===");
    let mut sorted: Vec<_> = pcs.iter().collect();
    sorted.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
    for (pc, count) in sorted.iter().take(10) {
        println!("  PC={:#010X} count={}", pc, count);
    }

    let max_vram_addr = log
        .iter()
        .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600C000)
        .map(|(a, _, _)| *a)
        .max()
        .unwrap_or(0);
    println!(
        "Max VRAM tile address written: {:#010X} (offset {:#X}, tile {})",
        max_vram_addr,
        max_vram_addr - 0x06000000,
        (max_vram_addr - 0x06000000) / 32
    );
}
