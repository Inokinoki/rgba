use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..10 {
        gba.run_frame_parallel(&mut fb);

        let m = gba.mem_mut();
        let handler_ptr = m.read_word(0x03007FFC);
        let vblank_ctr = m.read_word(0x03007FF8);
        let ie = m.interrupt.ie.bits();
        let if_ = m.interrupt.if_raw.bits();
        let ime = m.interrupt.ime;
        let pc = gba.cpu().get_pc();

        println!("F{}: handler=0x{:08X} vblank_ctr={} IE=0x{:04X} IF=0x{:04X} IME={} PC=0x{:08X}",
            frame, handler_ptr, vblank_ctr, ie, if_, ime, pc);
    }

    // Check if game's handler at 0x03000958 has code
    let m = gba.mem_mut();
    let h0 = m.read_word(0x03000958);
    let h1 = m.read_word(0x0300095C);
    println!("\nGame handler at 0x03000958: 0x{:08X} 0x{:08X}", h0, h1);
    println!("[0x03007FFC] = 0x{:08X}", m.read_word(0x03007FFC));
}
