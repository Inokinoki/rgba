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
    let iwram = gba.mem().iwram();
    let wram = gba.mem().wram();

    // Check pointer chain from VBlank callback
    let ptr1 = u32::from_le_bytes([iwram[0x0410], iwram[0x0411], iwram[0x0412], iwram[0x0413]]);
    println!("IWRAM[0x0410] = {:#010X}", ptr1);

    if ptr1 >= 0x02000000 && ptr1 < 0x03000000 {
        let off = (ptr1 - 0x02000000) as usize;
        let ptr2 = u32::from_le_bytes([wram[off], wram[off+1], wram[off+2], wram[off+3]]);
        println!("*{:#010X} = {:#010X}", ptr1, ptr2);
    } else if ptr1 >= 0x03000000 && ptr1 < 0x03008000 {
        let off = (ptr1 - 0x03000000) as usize;
        let ptr2 = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
        println!("*{:#010X} = {:#010X}", ptr1, ptr2);
    }

    // Disassemble main loop area at 0x08009910-0x08009940 (THUMB)
    println!("\n=== Main loop at 0x08009910 (THUMB) ===");
    for i in 0..30 {
        let off = (0x08009910 - 0x08000000 + i * 2) as usize;
        if off + 2 <= rom.len() {
            let half = u16::from_le_bytes([rom[off], rom[off+1]]);
            let addr = 0x08009910 + (i * 2) as u32;
            println!("  {:#010X}: {:#06X}", addr, half);
        }
    }

    // Disassemble VBlank processing function at 0x080D30C8 (THUMB)
    println!("\n=== VBlank process at 0x080D30C8 (THUMB) ===");
    for i in 0..40 {
        let off = (0x080D30C8 - 0x08000000 + i * 2) as usize;
        if off + 2 <= rom.len() {
            let half = u16::from_le_bytes([rom[off], rom[off+1]]);
            let addr = 0x080D30C8 + (i * 2) as u32;
            println!("  {:#010X}: {:#06X}", addr, half);
        }
    }

    // Disassemble 0x08009290 area (also hot)
    println!("\n=== Hot code at 0x08009290 (THUMB) ===");
    for i in 0..30 {
        let off = (0x08009290 - 0x08000000 + i * 2) as usize;
        if off + 2 <= rom.len() {
            let half = u16::from_le_bytes([rom[off], rom[off+1]]);
            let addr = 0x08009290 + (i * 2) as u32;
            println!("  {:#010X}: {:#06X}", addr, half);
        }
    }

    // Check the IntrWait wrapper at 0x080D2F10
    println!("\n=== IntrWait wrapper at 0x080D2F10 (THUMB) ===");
    for i in 0..20 {
        let off = (0x080D2F10 - 0x08000000 + i * 2) as usize;
        if off + 2 <= rom.len() {
            let half = u16::from_le_bytes([rom[off], rom[off+1]]);
            let addr = 0x080D2F10 + (i * 2) as u32;
            println!("  {:#010X}: {:#06X}", addr, half);
        }
    }

    // Also check: what does the game do between IntrWait calls?
    // Run one more frame with a full PC trace, sorted by address
    println!("\n=== Full execution flow (frame 8, sorted by PC) ===");
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x200000 / 2];

    gba.run_frame_parallel(&mut framebuffer);

    let counts = &gba.mem().pc_trace_counts;
    let mut pcs: Vec<(u32, u32)> = Vec::new();
    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            pcs.push((0x08000000 + (i as u32) * 2, count));
        }
    }
    pcs.sort_by_key(|(pc, _)| *pc);

    // Print first 100 to see the flow
    for (pc, count) in pcs.iter().take(100) {
        println!("  {:#010X}: {}", pc, count);
    }
}
