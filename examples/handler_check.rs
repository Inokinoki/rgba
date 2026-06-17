use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..10 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let iwram = gba.mem().iwram();

    // Check raw bytes at 0x7FFC
    let raw_bytes = [iwram[0x7FFC], iwram[0x7FFD], iwram[0x7FFE], iwram[0x7FFF]];
    let handler_ptr = u32::from_le_bytes(raw_bytes);
    println!("IWRAM[0x7FFC] raw bytes: {:02X} {:02X} {:02X} {:02X}", raw_bytes[0], raw_bytes[1], raw_bytes[2], raw_bytes[3]);
    println!("Handler pointer: {:#010X} (bit0={})", handler_ptr, handler_ptr & 1);

    // Dump handler code at 0x03000958 (both ARM and THUMB interpretation)
    let handler_offset = (0x03000958 - 0x03000000) as usize;
    println!("\n=== Code at 0x03000958 (ARM interpretation) ===");
    for i in 0..8 {
        let off = handler_offset + i * 4;
        if off + 4 <= iwram.len() {
            let word = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
            println!("  {:#010X}: {:#010X}", 0x03000958 + (i*4) as u32, word);
        }
    }

    println!("\n=== Code at 0x03000958 (THUMB interpretation) ===");
    for i in 0..16 {
        let off = handler_offset + i * 2;
        if off + 2 <= iwram.len() {
            let half = u16::from_le_bytes([iwram[off], iwram[off+1]]);
            println!("  {:#010X}: {:#06X}", 0x03000958 + (i*2) as u32, half);
        }
    }

    // Also check: what was our stub handler overwritten with?
    let stub_offset = (0x03007E00 - 0x03000000) as usize;
    println!("\n=== Our stub at 0x03007E00 (first 24 bytes) ===");
    for i in 0..6 {
        let off = stub_offset + i * 4;
        if off + 4 <= iwram.len() {
            let word = u32::from_le_bytes([iwram[off], iwram[off+1], iwram[off+2], iwram[off+3]]);
            println!("  {:#010X}: {:#010X}", 0x03007E00 + (i*4) as u32, word);
        }
    }

    // Check what mode the CPU is in during the handler
    // Let's also check: does the game write the handler pointer with thumb bit?
    // Search for writes to 0x03007FFC
    println!("\n=== Checking handler setup ===");

    // Run one more frame with PC trace including IWRAM
    let trace_base = 0x03000000;
    let trace_size = 0x8000; // 32K halfwords = 64KB (all IWRAM)
    gba.mem_mut().pc_trace_base = trace_base;
    gba.mem_mut().pc_trace_counts = vec![0u32; trace_size];

    gba.run_frame_parallel(&mut framebuffer);

    let counts = &gba.mem().pc_trace_counts;
    let mut iwram_pcs: Vec<(u32, u32)> = Vec::new();
    for (i, &count) in counts.iter().enumerate() {
        if count > 0 {
            iwram_pcs.push((trace_base + (i as u32) * 2, count));
        }
    }
    iwram_pcs.sort_by(|a, b| b.1.cmp(&a.1));

    println!("IWRAM PCs executed in frame 10:");
    for (pc, count) in iwram_pcs.iter().take(30) {
        println!("  {:#010X}: {}", pc, count);
    }
}
