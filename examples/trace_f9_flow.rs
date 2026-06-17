use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..8 { gba.run_frame_parallel(&mut fb); }

    // Now trace every instruction for one frame using run_scanline
    println!("=== Instruction trace for frame 9 ===");
    
    // Hook into the memory's pc_trace to capture ALL PCs
    gba.mem_mut().pc_trace_base = 0;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x10000000]; // covers all 32-bit address space / 2
    
    // Run one frame
    for _ in 0..228 {
        gba.run_scanline();
    }
    
    // Collect and sort PCs by first occurrence
    let trace = &gba.mem().pc_trace_counts;
    println!("Total trace entries (non-zero): {}", trace.iter().filter(|&&x| x > 0).count());
    
    // Print in order - BIOS region first
    for addr_div2 in 0..0x2000 {
        if trace[addr_div2] > 0 {
            let pc = (addr_div2 as u32) * 2;
            println!("0x{:08X}: {}", pc, trace[addr_div2]);
        }
    }
    
    // EWRAM
    for addr_div2 in (0x02000000/2)..(0x02040000/2) {
        if trace[addr_div2] > 0 {
            let pc = (addr_div2 as u32) * 2;
            println!("0x{:08X}: {}", pc, trace[addr_div2]);
        }
    }
    
    // IWRAM
    for addr_div2 in (0x03000000/2)..(0x03008000/2) {
        if trace[addr_div2] > 0 {
            let pc = (addr_div2 as u32) * 2;
            println!("0x{:08X}: {}", pc, trace[addr_div2]);
        }
    }
    
    // ROM region
    for addr_div2 in (0x08000000/2)..(0x08200000/2) {
        if trace[addr_div2] > 0 {
            let pc = (addr_div2 as u32) * 2;
            println!("0x{:08X}: {}", pc, trace[addr_div2]);
        }
    }
}
