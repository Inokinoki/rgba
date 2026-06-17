use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Check timer registers and IRQ state
    for frame in 0..=610 {
        gba.run_frame_parallel(&mut fb);
        if frame % 100 == 0 || (frame >= 560 && frame <= 575) {
            let tm0cnt = gba.mem.read_half(0x04000100);  // Timer 0 count
            let tm0cr = gba.mem.read_half(0x04000102);   // Timer 0 control
            let tm1cnt = gba.mem.read_half(0x04000104);  // Timer 1 count
            let tm1cr = gba.mem.read_half(0x04000106);   // Timer 1 control
            let tm2cnt = gba.mem.read_half(0x04000108);  // Timer 2 count
            let tm2cr = gba.mem.read_half(0x0400010A);   // Timer 2 control
            let tm3cnt = gba.mem.read_half(0x0400010C);  // Timer 3 count
            let tm3cr = gba.mem.read_half(0x0400010E);   // Timer 3 control
            let ie = gba.mem.read_half(0x04000200);      // Interrupt Enable
            let ime = gba.mem.read_half(0x04000208);     // Interrupt Master Enable
            let vblk = gba.mem.read_word(0x03007FF8);
            let val74 = gba.mem.read_word(0x02000074);
            println!("Frame {:3}: TM0={:04X}:{:04X} TM1={:04X}:{:04X} TM2={:04X}:{:04X} TM3={:04X}:{:04X} IE={:04X} IME={:04X} VBLK={:08X} ST74={:08X}",
                frame, tm0cnt, tm0cr, tm1cnt, tm1cr, tm2cnt, tm2cr, tm3cnt, tm3cr, ie, ime, vblk, val74);
        }
    }
}
