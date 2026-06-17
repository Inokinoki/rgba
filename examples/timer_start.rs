use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    let mut last_ctrl = 0u16;

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem.io();
        let t0_ctrl = u16::from_le_bytes([io[0x102], io[0x103]]);
        if t0_ctrl != last_ctrl {
            let t0_cnt = u16::from_le_bytes([io[0x100], io[0x101]]);
            drop(io);
            let state = gba.mem.read_word(0x02000074);
            println!("Frame {:4}: Timer0 CNT={:04X} CTRL={:04X} state={:08X}", frame, t0_cnt, t0_ctrl, state);
            last_ctrl = t0_ctrl;
        }
    }
}
