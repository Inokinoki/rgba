use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.mem.iwram_write_log_enabled = true;
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to frame 200
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Enable write logging for VBLK area
    gba.mem.iwram_write_log.clear();
    gba.mem.iwram_write_log_enabled = true;
    
    // Run one frame
    gba.run_frame_parallel(&mut fb);
    
    println!("IWRAM writes ({} total):", gba.mem.iwram_write_log.len());
    for (addr, pc, val) in gba.mem.iwram_write_log.iter().take(100) {
        if *addr >= 0x7FF0 && *addr <= 0x7FFF {
            println!("  [{:04X}] = {:02X} (PC={:08X})", addr, val, pc);
        }
    }
}
