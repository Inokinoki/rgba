use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.mem.swi_log_enabled = true;
    gba.mem.swi_log.clear();
    
    let mut fb = vec![0u32; 240 * 160];
    
    for frame in 0..=210 {
        gba.run_frame_parallel(&mut fb);
        let swi_04_05_count = gba.mem.swi_log.iter().filter(|&&x| x == 0x04 || x == 0x05).count();
        if frame <= 5 || frame % 50 == 0 {
            println!("Frame {:3}: total SWIs={} VBlankIntrWait={} last5={:?}",
                frame, 
                gba.mem.swi_log.len(),
                swi_04_05_count,
                &gba.mem.swi_log[gba.mem.swi_log.len().saturating_sub(5)..]);
        }
    }
}
