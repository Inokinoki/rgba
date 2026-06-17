use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 { gba.run_frame_parallel(&mut fb); }
    
    println!("ARM SWIs: {}, THUMB SWIs: {}", gba.mem.arm_swi_count, gba.mem.thumb_swi_count);
    println!("use_real_bios: {}", gba.mem.use_real_bios);
    
    // Set VBLK to 0x42 and run 1 frame with SWI logging
    {
        let iwram = gba.mem.iwram_mut();
        iwram[0x7FF8] = 0x42;
        iwram[0x7FF9] = 0x00;
        iwram[0x7FFA] = 0x00;
        iwram[0x7FFB] = 0x00;
    }
    gba.mem.swi_log_enabled = true;
    gba.mem.swi_log.clear();
    gba.mem.arm_swi_count = 0;
    gba.mem.thumb_swi_count = 0;
    
    gba.run_frame_parallel(&mut fb);
    
    println!("After 1 frame: VBLK={:08X}", gba.mem.read_word(0x03007FF8));
    println!("ARM SWIs: {}, THUMB SWIs: {}", gba.mem.arm_swi_count, gba.mem.thumb_swi_count);
    println!("SWI log: {:?}", gba.mem.swi_log);
}
