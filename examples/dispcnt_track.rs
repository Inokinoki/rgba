use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    let mut last_dispcnt = 0u16;
    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem.io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        if dispcnt != last_dispcnt {
            println!("Frame {:4}: DISPCNT {:04X} -> {:04X}", frame, last_dispcnt, dispcnt);
            last_dispcnt = dispcnt;
        }
    }
}
