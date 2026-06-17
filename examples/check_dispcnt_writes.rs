use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Enable DISPCNT write logging
    gba.mem_mut().dispcnt_write_log_enabled = true;

    let mut fb = vec![0u32; 240 * 160];
    for frame in 0..15 {
        gba.run_frame_parallel(&mut fb);
        
        let dispcnt = {
            let io = gba.mem().io();
            u16::from_le_bytes([io[0], io[1]])
        };
        
        let log = &gba.mem().dispcnt_write_log;
        let entries: Vec<_> = log.iter().collect();
        println!("Frame {}: DISPCNT=0x{:04X} writes={}", frame, dispcnt, entries.len());
        for e in &entries {
            println!("  PC=0x{:08X} off={} val=0x{:02X}", e.0, e.1, e.2);
        }
        gba.mem_mut().dispcnt_write_log.clear();
    }
}
