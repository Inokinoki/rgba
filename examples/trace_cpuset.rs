use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Enable CpuSet logging
    gba.mem_mut().cpu_set_log_enabled = true;
    gba.mem_mut().swi_log_enabled = true;
    
    let mut fb = vec![0u32; 240 * 160];
    
    for frame in 0..7 {
        gba.mem_mut().swi_log.clear();
        gba.run_frame_parallel(&mut fb);
        
        let dispcnt = {
            let io = gba.mem().io();
            u16::from_le_bytes([io[0], io[1]])
        };
        
        println!("Frame {}: DISPCNT=0x{:04X} SWIs={:?}", frame, dispcnt, 
            gba.mem().swi_log.iter().fold(std::collections::HashMap::new(), |mut acc, &n| {
                *acc.entry(n).or_insert(0) += 1;
                acc
            }));
    }
    
    println!("\n=== CpuSet calls (first 10) ===");
    for (i, &(src, dst, cnt)) in gba.mem().cpu_set_log.iter().take(10).enumerate() {
        let fill = (cnt >> 24) & 1 != 0;
        let count = cnt & 0x1FFFFF;
        let is_32 = (cnt >> 26) & 1 != 0;
        let size = if is_32 { count * 4 } else { count * 2 };
        println!("{}: src=0x{:08X} dst=0x{:08X} cnt=0x{:08X} ({}count={} {} {})", 
            i, src, dst, cnt,
            if fill { "FILL " } else { "" },
            count,
            if is_32 { "32bit" } else { "16bit" },
            size);
    }
    
    // Also check the VBlank counter at IWRAM 0x7FF8
    let iwram = gba.mem().iwram();
    let counter_lo = u16::from_le_bytes([iwram[0x7FF8], iwram[0x7FF9]]);
    let counter_hi = u16::from_le_bytes([iwram[0x7FFA], iwram[0x7FFB]]);
    let counter = ((counter_hi as u32) << 16) | counter_lo as u32;
    println!("\nVBlank counter at IWRAM 0x7FF8: {}", counter);
    
    // Check the game's state variables
    // The game uses a state variable to track progression
    // Let me check common locations
    println!("\n=== Key IWRAM values ===");
    for offset in (0..0x200).step_by(4) {
        let val = u32::from_le_bytes([iwram[offset], iwram[offset+1], iwram[offset+2], iwram[offset+3]]);
        if val != 0 && (val < 0x100 || val > 0x80000000) {
            println!("IWRAM[0x{:04X}]: 0x{:08X} ({})", offset, val, val);
        }
    }
}
