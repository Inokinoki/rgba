use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    // Run 6 frames
    for _ in 0..6 { gba.run_frame_parallel(&mut fb); }

    // Trace frame 7 - count all PCs in ROM
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x200000]; // 0x08000000-0x08400000

    // Also trace IWRAM
    // We can only trace one range. Let's focus on ROM first.
    
    gba.run_frame_parallel(&mut fb);
    
    let trace = &gba.mem().pc_trace_counts;
    let total: u32 = trace.iter().sum();
    
    let dispcnt = {
        let io = gba.mem().io();
        u16::from_le_bytes([io[0], io[1]])
    };
    
    println!("Frame 7: total ROM PC hits={}, DISPCNT=0x{:04X}", total, dispcnt);
    
    // Show top 30 PCs
    let mut entries: Vec<(usize, u32)> = trace.iter().cloned().enumerate().filter(|&(_, v)| v > 0).collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    
    let rom_data: Vec<u8> = gba.mem().rom().to_vec();
    
    println!("\nTop 30 PCs:");
    for (idx, count) in entries.iter().take(30) {
        let pc = 0x08000000 + (idx * 2) as u32;
        let off = (pc - 0x08000000) as usize;
        let half = if off + 1 < rom_data.len() {
            u16::from_le_bytes([rom_data[off], rom_data[off+1]])
        } else { 0 };
        println!("  0x{:08X}: {} hits, opcode=0x{:04X}", pc, count, half);
    }
    
    // Show all unique PCs in order
    println!("\nAll unique PCs in address order (first 100):");
    let mut count2 = 0;
    for (idx, &cnt) in trace.iter().enumerate() {
        if cnt > 0 {
            let pc = 0x08000000 + (idx * 2) as u32;
            let off = (pc - 0x08000000) as usize;
            let half = if off + 1 < rom_data.len() {
                u16::from_le_bytes([rom_data[off], rom_data[off+1]])
            } else { 0 };
            println!("  0x{:08X}: {} (0x{:04X})", pc, cnt, half);
            count2 += 1;
            if count2 >= 100 { break; }
        }
    }
}
