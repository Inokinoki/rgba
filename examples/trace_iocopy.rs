use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Trace specific addresses
    gba.mem_mut().pc_trace_base = 0x08000000;
    gba.mem_mut().pc_trace_counts = vec![0u32; 0x100000]; // 0x08000000-0x08200000

    let mut fb = vec![0u32; 240 * 160];
    for frame in 0..20 {
        // Clear trace each frame
        for c in gba.mem_mut().pc_trace_counts.iter_mut() { *c = 0; }
        
        gba.run_frame_parallel(&mut fb);
        
        let dispcnt = {
            let io = gba.mem().io();
            u16::from_le_bytes([io[0], io[1]])
        };
        
        let trace = &gba.mem().pc_trace_counts;
        let total: u32 = trace.iter().sum();
        
        // Check specific addresses
        let check_addr = |addr: u32| -> u32 {
            let idx = ((addr - 0x08000000) / 2) as usize;
            trace[idx]
        };
        
        let f_d6fc6 = check_addr(0x080D6FC6);
        let f_d6fe6 = check_addr(0x080D6FE6);
        let f_08e6a = check_addr(0x08008E6A);
        let f_096f6 = check_addr(0x080096F6);
        let f_d30ca = check_addr(0x080D30CA);
        
        println!("Frame {}: DISPCNT=0x{:04X} total={} | d6fc6={} d6fe6={} 08e6a={} 096f6={} d30ca={}",
            frame, dispcnt, total, f_d6fc6, f_d6fe6, f_08e6a, f_096f6, f_d30ca);
    }
}
