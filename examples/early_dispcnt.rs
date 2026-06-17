use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    let mut last = 0xFFFFu16;
    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        if dispcnt != last || frame < 10 {
            println!("Frame {:3}: DISPCNT=0x{:04X}", frame, dispcnt);
            last = dispcnt;
        }
    }
}
