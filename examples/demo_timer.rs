use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..580 {
        gba.run_frame_parallel(&mut fb);
        if frame >= 190 && frame <= 200 || frame >= 560 && frame <= 575 {
            let v50 = gba.mem.read_word(0x02000050);
            let state = gba.mem.read_word(0x02000074);
            let v90 = gba.mem.read_word(0x02000090);
            let v68 = gba.mem.read_word(0x02000068);
            let va0 = gba.mem.read_word(0x020000A0);
            let vf0 = gba.mem.read_word(0x020000F0);
            println!(
                "Frame {:4}: [50]={:08X} [68]={:08X} [74]={:08X} [90]={:08X} [A0]={:08X} [F0]={:08X}",
                frame, v50, v68, state, v90, va0, vf0
            );
        }
    }
}
