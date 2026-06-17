use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Run to frame 500
    for _ in 0..500 { gba.run_frame_parallel(&mut fb); }
    
    // Now check every frame from 500 to 600
    for frame in 500..=610 {
        gba.run_frame_parallel(&mut fb);
        let val74 = gba.mem.read_word(0x02000074);
        let val7c = gba.mem.read_word(0x0200007C);
        let val60 = gba.mem.read_word(0x02000060);
        let val64 = gba.mem.read_word(0x02000064);
        let dispcnt = gba.ppu().get_dispcnt();
        let keyinput = gba.mem.read_half(0x04000130);
        if val74 != 0x00000001 || val7c != 0x00000000 || frame % 20 == 0 {
            println!("Frame {:3}: [0074]={:08X} [007C]={:08X} [0060]={:08X} [0064]={:08X} DISPCNT={:04X} KEYINPUT={:04X}",
                frame, val74, val7c, val60, val64, dispcnt, keyinput);
        }
    }
}
