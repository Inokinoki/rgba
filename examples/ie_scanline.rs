use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..7 {
        for scanline in 0..228 {
            gba.run_scanline();

            let io = gba.mem().io();
            let ie = u16::from_le_bytes([io[0x200], io[0x201]]);
            let ime = io[0x208];
            let halted = gba.cpu().is_halted();

            if ie != 0 || !halted || ime != 0 {
                if frame <= 6 {
                    println!(
                        "Frame {} SL {:3}: IE={:04X} IME={:02X} halted={}",
                        frame, scanline, ie, ime, halted
                    );
                }
            }
        }
    }
}
