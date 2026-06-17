use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..7 { gba.run_frame_parallel(&mut fb); }

    // Now trace frame 7 in detail
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x100000];
    
    gba.run_frame_parallel(&mut fb);
    
    let trace = &gba.mem().pc_trace_counts;
    let rom = gba.mem().rom();
    
    // Show every instruction from 0x080D6FC0 to 0x080D7020
    println!("=== Code at 0x080D6FC0-0x080D7020 ===");
    for addr in (0x080D6FC0..=0x080D7020).step_by(2) {
        let idx = ((addr - 0x08000000) / 2) as usize;
        let cnt = trace[idx];
        let off = (addr - 0x08000000) as usize;
        let half = u16::from_le_bytes([rom[off], rom[off+1]]);
        println!("0x{:08X}: {} (0x{:04X})", addr, cnt, half);
    }
    
    // Also show 0x08009700-0x08009720 (the IO copy setup)
    println!("\n=== Code at 0x080096F0-0x08009720 ===");
    for addr in (0x080096F0..=0x08009720).step_by(2) {
        let idx = ((addr - 0x08000000) / 2) as usize;
        let cnt = trace[idx];
        let off = (addr - 0x08000000) as usize;
        let half = u16::from_le_bytes([rom[off], rom[off+1]]);
        println!("0x{:08X}: {} (0x{:04X})", addr, cnt, half);
    }
    
    // Show 0x08008E60-0x08008EC0 (the IO copy function)
    println!("\n=== Code at 0x08008E60-0x08008EC0 ===");
    for addr in (0x08008E60..=0x08008EC0).step_by(2) {
        let idx = ((addr - 0x08000000) / 2) as usize;
        let cnt = trace[idx];
        let off = (addr - 0x08000000) as usize;
        let half = u16::from_le_bytes([rom[off], rom[off+1]]);
        println!("0x{:08X}: {} (0x{:04X})", addr, cnt, half);
    }
}
