use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Check EWRAM at early frames
    for checkpoint in [1, 5, 10, 20, 50, 100, 200, 300, 400, 500, 600] {
        for _ in 0..checkpoint { gba.run_frame_parallel(&mut fb); }
        
        let val0 = gba.mem.read_word(0x02000000);
        let val4 = gba.mem.read_word(0x02000004);
        let val74 = gba.mem.read_word(0x02000074);
        let val7c = gba.mem.read_word(0x0200007C);
        let valc0 = gba.mem.read_word(0x020000C0);
        let valf0 = gba.mem.read_word(0x020000F0);
        println!("Frame {:3}: [{:08X}]={:08X} [{:08X}]={:08X} [{:08X}]={:08X} [{:08X}]={:08X} [{:08X}]={:08X} [{:08X}]={:08X}",
            checkpoint, 
            0x02000000u32, val0,
            0x02000004u32, val4,
            0x02000074u32, val74,
            0x0200007Cu32, val7c,
            0x020000C0u32, valc0,
            0x020000F0u32, valf0);
    }
}
