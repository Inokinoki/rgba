use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.mem.swi_log_enabled = true;
    gba.mem.swi_log.clear();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    // Set VBLK to a known value
    {
        let iwram = gba.mem.iwram_mut();
        iwram[0x7FF8] = 0x42;
        iwram[0x7FF9] = 0x00;
        iwram[0x7FFA] = 0x00;
        iwram[0x7FFB] = 0x00;
    }
    gba.mem.swi_log.clear();
    
    for frame in 0..5 {
        let before = gba.mem.read_word(0x03007FF8);
        gba.run_frame_parallel(&mut fb);
        let after = gba.mem.read_word(0x03007FF8);
        let swi_04_count = gba.mem.swi_log.iter().filter(|&&x| x == 0x04).count();
        println!("Frame {}: VBLK {:08X} -> {:08X} SWI04={}", frame, before, after, swi_04_count);
        gba.mem.swi_log.clear();
    }
}
