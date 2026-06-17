use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for _scanline in 0..228 {
            gba.run_scanline();
        }

        if frame == 190 || frame == 195 || frame == 199 {
            let io = gba.mem().io();
            // DMA0: 0xB0-BF, DMA1: 0xC0-CF, DMA2: 0xD0-DF, DMA3: 0xE0-EF
            for ch in 0..4 {
                let base = 0xB0 + ch * 0x10;
                let src = u32::from_le_bytes([io[base], io[base + 1], io[base + 2], io[base + 3]]);
                let dst =
                    u32::from_le_bytes([io[base + 4], io[base + 5], io[base + 6], io[base + 7]]);
                let cnt = u16::from_le_bytes([io[base + 0xA], io[base + 0xB]]);
                let word_cnt = u16::from_le_bytes([io[base + 8], io[base + 9]]);

                let enabled = (cnt >> 15) & 1;
                let timing = (cnt >> 12) & 3;
                let timing_name = match timing {
                    0 => "Immediate",
                    1 => "VBlank",
                    2 => "HBlank",
                    3 => "Special",
                    _ => "?",
                };
                let repeat = (cnt >> 9) & 1;
                let dst_ctrl = (cnt >> 10) & 3;
                let src_ctrl = (cnt >> 7) & 3;
                let irq = (cnt >> 14) & 1;

                if enabled != 0 {
                    println!("Frame {} DMA{}: src={:08X} dst={:08X} cnt={:04X} wc={:04X} timing={} repeat={} dst_ctrl={} src_ctrl={} irq={}", 
                             frame, ch, src, dst, cnt, word_cnt, timing_name, repeat, dst_ctrl, src_ctrl, irq);
                }
            }
        }
    }
}
