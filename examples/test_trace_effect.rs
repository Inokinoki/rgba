use rgba::Gba;

fn main() {
    // Test 1: No tracing at all
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];
    
    for frame in 0..10 {
        gba.mem_mut().swi_log_enabled = true;
        gba.mem_mut().swi_log.clear();
        gba.run_frame_parallel(&mut fb);
        
        let dispcnt = {
            let io = gba.mem().io();
            u16::from_le_bytes([io[0], io[1]])
        };
        let swi_count = gba.mem().swi_log.len();
        
        // For frame 7, also check total instructions via separate trace
        if frame == 7 {
            println!("Test1 (no trace) F{}: DISPCNT=0x{:04X} SWIs={:?} swi_count={}",
                frame, dispcnt,
                gba.mem().swi_log.iter().fold(std::collections::HashMap::new(), |mut acc, &n| {
                    *acc.entry(n).or_insert(0) += 1;
                    acc
                }),
                swi_count);
        }
    }
    
    // Now check: at frame 10, what's DISPCNT?
    let dispcnt = {
        let io = gba.mem().io();
        u16::from_le_bytes([io[0], io[1]])
    };
    println!("Test1 after F9: DISPCNT=0x{:04X}", dispcnt);
    
    // Test 2: With tracing every frame
    let mut gba2 = Gba::new();
    gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    
    for frame in 0..10 {
        gba2.mem_mut().pc_trace_base = 0x080C0000;
        gba2.mem_mut().pc_trace_counts = vec![0u32; 0x20000];
        gba2.mem_mut().swi_log_enabled = true;
        gba2.mem_mut().swi_log.clear();
        gba2.run_frame_parallel(&mut fb);
        
        let dispcnt = {
            let io = gba2.mem().io();
            u16::from_le_bytes([io[0], io[1]])
        };
        let trace_total: u32 = gba2.mem().pc_trace_counts.iter().sum();
        
        if frame >= 5 && frame <= 9 {
            println!("Test2 (with trace) F{}: DISPCNT=0x{:04X} trace_total={} SWIs={:?}",
                frame, dispcnt, trace_total,
                gba2.mem().swi_log.iter().fold(std::collections::HashMap::new(), |mut acc, &n| {
                    *acc.entry(n).or_insert(0) += 1;
                    acc
                }));
        }
    }
    
    let dispcnt2 = {
        let io = gba2.mem().io();
        u16::from_le_bytes([io[0], io[1]])
    };
    println!("Test2 after F9: DISPCNT=0x{:04X}", dispcnt2);
    
    // Compare EWRAM state
    println!("\n=== EWRAM comparison (first differences) ===");
    let mut diffs = 0;
    for addr in (0x02000000..0x02040000u32).step_by(4) {
        let v1 = gba.mem_mut().read_word(addr);
        let v2 = gba2.mem_mut().read_word(addr);
        if v1 != v2 {
            println!("0x{:08X}: test1=0x{:08X} test2=0x{:08X}", addr, v1, v2);
            diffs += 1;
            if diffs >= 20 { break; }
        }
    }
    if diffs == 0 { println!("EWRAM matches!"); }
    
    // Compare IWRAM state
    println!("\n=== IWRAM comparison (first differences) ===");
    let mut diffs = 0;
    for addr in (0x03000000..0x03007F00u32).step_by(4) {
        let v1 = gba.mem_mut().read_word(addr);
        let v2 = gba2.mem_mut().read_word(addr);
        if v1 != v2 {
            println!("0x{:08X}: test1=0x{:08X} test2=0x{:08X}", addr, v1, v2);
            diffs += 1;
            if diffs >= 20 { break; }
        }
    }
    if diffs == 0 { println!("IWRAM matches!"); }
}
