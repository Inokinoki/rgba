use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..10 {
        for scanline in 0..228 {
            let ie_before = gba.mem().interrupt.ie.bits();
            let if_before = gba.mem().interrupt.if_raw.bits();

            gba.run_scanline();

            let ie_after = gba.mem().interrupt.ie.bits();
            let if_after = gba.mem().interrupt.if_raw.bits();

            // Show VBlank scanline transitions
            if (scanline >= 158 && scanline <= 162) || if_before != if_after {
                let halted = gba.cpu().is_halted();
                println!(
                    "Frame {} SL {:3}: IE {:04X}->{:04X} IF {:04X}->{:04X} halt={}",
                    frame, scanline, ie_before, ie_after, if_before, if_after, halted
                );
            }
        }
    }
}
