use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    let mut last_state = 0xFFFFFFFFu32;
    for frame in 0..1200 {
        gba.run_frame_parallel(&mut fb);

        let io = gba.mem().io();
        let state = u32::from_le_bytes([io[0x74], io[0x75], io[0x76], io[0x77]]);
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let bg0hofs = u16::from_le_bytes([io[0x10], io[0x11]]) & 0x1FF;

        if state != last_state || frame % 100 == 0 {
            println!(
                "Frame {:4}: state=0x{:08X} dispcnt=0x{:04X} hofs={}",
                frame, state, dispcnt, bg0hofs
            );
            last_state = state;
        }
    }
}
