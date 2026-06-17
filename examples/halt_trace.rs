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
        let ime = io[0x208];
        let halted = gba.cpu().is_halted();

        if halted || ie == 0 || frame >= 185 {
            let dispcnt = u16::from_le_bytes([io[0], io[1]]);
            println!(
                "Frame {:3}: DISPCNT={:04X} IE={:04X} IME={:02X} halted={}",
                frame, dispcnt, ie, ime, halted
            );
        }
    }
}
