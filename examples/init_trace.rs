use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..800 {
        let prev_dispcnt = u16::from_le_bytes([gba.mem.io()[0], gba.mem.io()[1]]);
        gba.run_frame_parallel(&mut fb);
        let dispcnt = u16::from_le_bytes([gba.mem.io()[0], gba.mem.io()[1]]);

        if frame >= 185 && frame <= 200 {
            let state = gba.mem.read_word(0x02000074);
            let vblk = gba.mem.read_word(0x03007FF8);
            let pc = gba.cpu.get_pc();
            let mode = gba.cpu.get_mode();
            let halted = gba.cpu.is_halted();
            let ime = gba.mem.interrupt.ime;
            println!("Frame {:4}: DISPCNT={:04X} State={:08X} VBLK={:08X} PC={:08X} mode={:?} halted={} IME={}", 
                     frame, dispcnt, state, vblk, pc, mode, halted, ime);
        }
    }
}
