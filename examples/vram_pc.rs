use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for _ in 0..7 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    let log = &gba.mem().vram_write_log;

    // Analyze unique PCs writing to BG tile area
    let bg_writes: Vec<_> = log
        .iter()
        .filter(|(addr, _, _)| *addr >= 0x06000000 && *addr < 0x0600F000)
        .collect();

    println!("BG tile area writes: {}", bg_writes.len());

    // Group by PC
    let mut pc_counts: std::collections::HashMap<u32, (usize, u8)> =
        std::collections::HashMap::new();
    for &&(addr, pc, val) in &bg_writes {
        let entry = pc_counts.entry(pc).or_insert((0, val));
        entry.0 += 1;
    }
    println!("\nUnique PCs writing to BG tile area:");
    for (pc, (count, sample_val)) in pc_counts.iter() {
        println!(
            "  PC={:08X}: {} writes (sample val={:02X})",
            pc, count, sample_val
        );
    }

    // Check what ROM code is at those PCs (THUMB mode)
    let rom = gba.mem().rom();
    for pc in pc_counts.keys() {
        let offset = (pc & !1) - 0x08000000;
        if offset < rom.len() as u32 - 20 {
            println!("\n  Code near PC {:08X}:", pc);
            for i in 0..10 {
                let instr_offset = offset as usize + i * 2;
                if instr_offset + 2 <= rom.len() {
                    let instr = u16::from_le_bytes([rom[instr_offset], rom[instr_offset + 1]]);
                    println!(
                        "    {:08X}: {:04X}",
                        0x08000000 + instr_offset as u32,
                        instr
                    );
                }
            }
        }
    }

    // Dump VRAM content at 0x06000000 (first 256 bytes = 8 tiles)
    let vram = gba.mem().vram();
    println!("\n=== VRAM content at 0x06000000 (first 256 bytes) ===");
    for row in 0..16 {
        let offset = row * 16;
        print!("  {:04X}: ", offset);
        for col in 0..16 {
            print!("{:02X} ", vram[offset + col]);
        }
        println!();
    }

    // Check the EWRAM source data that should have been loaded
    // DMA at frame 5 copies from 0x03007E3C to screen block 0x0600F800
    // But what about tile data? It should come from somewhere
    let wram = gba.mem().wram();
    println!("\n=== EWRAM at source addresses ===");
    // Check what data is at the EWRAM addresses used by CpuSet fills
    for addr in [0x02000C80, 0x02001CC0, 0x02000FD0, 0x02008D2C] {
        let offset = (addr - 0x02000000) as usize;
        print!("  {:08X}: ", addr);
        if offset + 32 <= wram.len() {
            for b in &wram[offset..offset + 32] {
                print!("{:02X} ", b);
            }
        }
        println!();
    }
}
