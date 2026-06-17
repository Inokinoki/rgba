use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }
    
    // Now trace frame 8 in detail - focusing on VBlank callback
    // Set up trace covering the callback and main loop code
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x100000]; // 0x08000000-0x08200000

    gba.run_frame_parallel(&mut fb);
    
    let trace = &gba.mem().pc_trace_counts;
    let total: u32 = trace.iter().sum();
    
    let dispcnt = {
        let io = gba.mem().io();
        u16::from_le_bytes([io[0], io[1]])
    };
    
    println!("Frame 8: total={} DISPCNT=0x{:04X}", total, dispcnt);
    
    // Show ALL PCs that executed, in order
    let rom_data: Vec<u8> = gba.mem().rom().to_vec();
    
    // Focus on the VBlank callback path
    // Show 0x080D13xx area (where the Smsh check happens)
    println!("\n=== Code at 0x080D13AE-0x080D1440 (callback function) ===");
    for addr_div2 in (0x080D13AE/2)..(0x080D1440/2) {
        let cnt = trace[addr_div2 - 0x08000000/2];
        let pc = (addr_div2 * 2) as u32;
        let off = (pc - 0x08000000) as usize;
        let half = if off + 1 < rom_data.len() {
            u16::from_le_bytes([rom_data[off], rom_data[off+1]])
        } else { 0 };
        if cnt > 0 {
            println!("0x{:08X}: {} (0x{:04X})", pc, cnt, half);
        }
    }
    
    // Also check the linked list traversal function at 0x08009910
    println!("\n=== Code at 0x08009910-0x08009950 (linked list traversal) ===");
    for addr_div2 in (0x08009910/2)..(0x08009950/2) {
        let cnt = trace[addr_div2 - 0x08000000/2];
        let pc = (addr_div2 * 2) as u32;
        let off = (pc - 0x08000000) as usize;
        let half = if off + 1 < rom_data.len() {
            u16::from_le_bytes([rom_data[off], rom_data[off+1]])
        } else { 0 };
        if cnt > 0 {
            println!("0x{:08X}: {} (0x{:04X})", pc, cnt, half);
        }
    }
    
    // Check the code that calls the linked list (around 0x08008B14)
    println!("\n=== Code at 0x08008B14-0x08008B40 (callback caller) ===");
    for addr_div2 in (0x08008B14/2)..(0x08008B40/2) {
        let cnt = trace[addr_div2 - 0x08000000/2];
        let pc = (addr_div2 * 2) as u32;
        let off = (pc - 0x08000000) as usize;
        let half = if off + 1 < rom_data.len() {
            u16::from_le_bytes([rom_data[off], rom_data[off+1]])
        } else { 0 };
        if cnt > 0 {
            println!("0x{:08X}: {} (0x{:04X})", pc, cnt, half);
        }
    }
    
    // Check for any writes to 0x04000000 area
    // Look for STR instructions targeting IO addresses
    println!("\n=== Any IO writes in 0x080D25xx area? ===");
    for addr_div2 in (0x080D25C0/2)..(0x080D2610/2) {
        let cnt = trace[addr_div2 - 0x08000000/2];
        let pc = (addr_div2 * 2) as u32;
        let off = (pc - 0x08000000) as usize;
        let half = if off + 1 < rom_data.len() {
            u16::from_le_bytes([rom_data[off], rom_data[off+1]])
        } else { 0 };
        if cnt > 0 {
            println!("0x{:08X}: {} (0x{:04X})", pc, cnt, half);
        }
    }
    
    // Let me also check the full list of unique code blocks that run
    println!("\n=== All code blocks with >3 hits ===");
    let mut prev_addr = 0u32;
    for addr_div2 in 0..trace.len() {
        if trace[addr_div2] > 3 {
            let pc = 0x08000000 + (addr_div2 as u32) * 2;
            if pc > prev_addr + 32 {
                println!();
            }
            print!("0x{:08X}:{}  ", pc, trace[addr_div2]);
            prev_addr = pc;
        }
    }
    println!();
}
