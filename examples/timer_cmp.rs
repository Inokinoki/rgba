use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Check timer state every 50 frames
    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);
        if frame % 50 == 0 || frame == 568 {
            let io = gba.mem.io();
            for t in 0..4 {
                let base = 0x100 + t * 4;
                let cnt_l = u16::from_le_bytes([io[base], io[base + 1]]);
                let cnt_h = u16::from_le_bytes([io[base + 2], io[base + 3]]);
                println!(
                    "Frame {:4} Timer{}: CNT={:04X} CTRL={:04X}",
                    frame, t, cnt_l, cnt_h
                );
            }
        }
    }
}
