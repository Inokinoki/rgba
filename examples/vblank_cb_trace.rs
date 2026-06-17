use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..8 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let rom = gba.mem().rom();

    // Disassemble the VBlank callback at 0x080D7098 (THUMB)
    println!("=== VBlank callback at 0x080D7098 (THUMB) ===");
    let cb_off = 0x080D7098 - 0x08000000;
    for i in 0..30 {
        let off = cb_off as usize + i * 2;
        if off + 2 <= rom.len() {
            let half = u16::from_le_bytes([rom[off], rom[off+1]]);
            let addr = 0x080D7098 + (i * 2) as u32;
            println!("  {:#010X}: {:#06X}", addr, half);
        }
    }

    // Also check what's at the "no interrupt found" exit path
    // and what IWRAM[0x7FF8] is used for
    println!("\n=== IWRAM[0x7FF0..0x8000] ===");
    let iwram = gba.mem().iwram();
    for i in 0..8 {
        let off = 0x7FF0 + i * 2;
        let half = u16::from_le_bytes([iwram[off], iwram[off+1]]);
        println!("  {:#010X}: {:#06X} ({})", 0x03000000 + off, half, half);
    }

    // Compare our state with what mGBA should have
    // Check key EWRAM values that might affect game progression
    println!("\n=== Key EWRAM state ===");
    let wram = gba.mem().wram();

    // Check if there are any non-zero timer values
    let io = gba.mem().io();
    println!("Timer0: data={:#04X}{:04X} ctrl={:#04X}{:04X}",
        io[0x101], io[0x100], io[0x103], io[0x102]);
    println!("Timer1: data={:#04X}{:04X} ctrl={:#04X}{:04X}",
        io[0x105], io[0x104], io[0x107], io[0x106]);
    println!("Timer2: data={:#04X}{:04X} ctrl={:#04X}{:04X}",
        io[0x109], io[0x108], io[0x10B], io[0x10A]);
    println!("Timer3: data={:#04X}{:04X} ctrl={:#04X}{:04X}",
        io[0x10D], io[0x10C], io[0x10F], io[0x10E]);

    // Let me trace ROM PCs during frame 8 to see if the VBlank callback executes
    println!("\n=== ROM trace for frame 8 ===");
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x200000 / 2]; // 1MB range

    gba.run_frame_parallel(&mut framebuffer);

    let counts = &gba.mem().pc_trace_counts;
    let mut rom_pcs: Vec<(u32, u32)> = Vec::new();
    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            rom_pcs.push((0x08000000 + (i as u32) * 2, count));
        }
    }
    rom_pcs.sort_by(|a, b| b.1.cmp(&a.1));

    let total: u32 = rom_pcs.iter().map(|(_, c)| *c).sum();
    println!("Total ROM instructions: {}", total);
    println!("Unique ROM PCs: {}", rom_pcs.len());

    println!("\nTop 30 ROM PCs:");
    for (pc, count) in rom_pcs.iter().take(30) {
        let in_callback = if *pc >= 0x080D7098 && *pc < 0x080D7100 {
            " ← VBlank callback area"
        } else if *pc >= 0x080D2F00 && *pc < 0x080D3000 {
            " ← IntrWait area"
        } else if *pc >= 0x08009900 && *pc < 0x08009A00 {
            " ← game code area"
        } else {
            ""
        };
        println!("  {:#010X}: {}{}", pc, count, in_callback);
    }
}
