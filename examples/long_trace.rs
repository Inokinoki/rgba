use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..400 {
        gba.run_frame_parallel(&mut fb);

        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let state = gba.mem_mut().read_word(0x02000074);
        let pc = gba.cpu().get_pc();

        if frame < 10 || frame % 50 == 0 || dispcnt != 0x0080 {
            println!("F{:3}: DISPCNT=0x{:04X} state={} PC=0x{:08X}",
                frame, dispcnt, state, pc);
        }
    }
}
