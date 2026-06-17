use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..8 {
        gba.run_frame_parallel(&mut fb);

        let mode = gba.cpu().get_mode();
        let pc = gba.cpu().get_pc();
        let irq_count = gba.cpu().irq_save_count;
        let m = gba.mem_mut();
        let vblank_ctr = m.read_word(0x03007FF8);
        let in_int = m.interrupt.in_interrupt;
        let if_ = m.interrupt.if_raw.bits();
        let ie = m.interrupt.ie.bits();

        println!("F{}: irq_count={} vblank_ctr={} in_int={} mode={:?} PC=0x{:08X} IE=0x{:04X} IF=0x{:04X}",
            frame, irq_count, vblank_ctr, in_int, mode, pc, ie, if_);
    }
}
