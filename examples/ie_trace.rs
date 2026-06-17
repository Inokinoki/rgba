use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for _scanline in 0..228 {
            gba.run_scanline();
        }

        let io = gba.mem().io();
        let ie = u16::from_le_bytes([io[0x200], io[0x201]]);
        let if_ = u16::from_le_bytes([io[0x202], io[0x203]]);
        let ime = io[0x208];
        let halted = gba.cpu().is_halted();

        if frame <= 6 || (frame >= 186 && frame <= 192) {
            println!(
                "Frame {:3}: IE={:04X} IF={:04X} IME={:02X} halted={}",
                frame, ie, if_, ime, halted
            );
        }
    }
}
