use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..60 {
        // Trace each frame briefly
        gba.mem_mut().pc_trace_base = 0x080C0000;
        gba.mem_mut().pc_trace_counts = vec![0u32; 0x20000];
        gba.mem_mut().swi_log_enabled = true;
        gba.mem_mut().swi_log.clear();

        gba.run_frame_parallel(&mut fb);

        let trace = &gba.mem().pc_trace_counts;
        let total: u32 = trace.iter().sum();
        let dispcnt = u16::from_le_bytes([gba.mem().io()[0], gba.mem().io()[1]]);
        
        let swi_summary: std::collections::HashMap<u32, usize> = 
            gba.mem().swi_log.iter().fold(std::collections::HashMap::new(), |mut acc, &n| {
                *acc.entry(n).or_insert(0) += 1;
                acc
            });

        if frame < 15 || frame % 10 == 0 || total > 1000 {
            println!("F{:2}: total_PC_hits={:6} DISPCNT=0x{:04X} SWIs={:?}",
                frame, total, dispcnt, swi_summary);
        }
    }
}
