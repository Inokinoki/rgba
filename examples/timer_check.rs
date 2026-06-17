use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for _scanline in 0..228 {
            gba.run_scanline();
        }

        if frame == 190 || frame == 199 {
            let io = gba.mem().io();
            // Timers: TM0CNT at 0x100-0x107, TM1 at 0x108-0x10F, etc.
            for t in 0..4 {
                let base = 0x100 + t * 4;
                let count = u16::from_le_bytes([io[base], io[base + 1]]);
                let control = io[base + 2];
                let enabled = (control >> 7) & 1;
                let irq_en = (control >> 6) & 1;
                let cascade = (control >> 2) & 1;
                let prescale = control & 3;
                if enabled != 0 {
                    println!(
                        "Frame {} TM{}: count={:04X} ctrl={:02X} irq={} cascade={} prescale={}",
                        frame, t, count, control, irq_en, cascade, prescale
                    );
                }
            }

            // Also check DISPCNT and DISPSTAT
            let dispcnt = u16::from_le_bytes([io[0], io[1]]);
            let dispstat = u16::from_le_bytes([io[4], io[5]]);
            let ie = u16::from_le_bytes([io[0x200], io[0x201]]);
            let ime = io[0x208];
            println!(
                "  DISPCNT={:04X} DISPSTAT={:04X} IE={:04X} IME={:02X}",
                dispcnt, dispstat, ie, ime
            );
        }
    }
}
