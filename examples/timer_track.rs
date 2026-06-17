use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..600 {
        gba.run_frame_parallel(&mut fb);
        if frame % 50 == 0 || frame == 194 {
            let v00 = gba.mem.read_word(0x02000000);
            let v50 = gba.mem.read_word(0x02000050);
            let va0 = gba.mem.read_word(0x020000A0);
            let vf0 = gba.mem.read_word(0x020000F0);
            let state = gba.mem.read_word(0x02000074);
            println!(
                "Ours frame {:4}: [00]={:08X} [50]={:08X} [A0]={:08X} [F0]={:08X} state={:08X}",
                frame, v00, v50, va0, vf0, state
            );
        }
    }
}
