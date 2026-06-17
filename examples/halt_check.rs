use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut last_pc = 0u32;
    let mut last_pc2 = 0u32;

    for frame in 0..400 {
        for _scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            // Track what code the CPU is executing
            last_pc2 = last_pc;
            last_pc = pc;
            gba.run_scanline();
        }

        if frame >= 190 && frame <= 210 {
            // Show where the CPU spends most time
            let io = gba.mem().io();
            let dispcnt = u16::from_le_bytes([io[0], io[1]]);
            let ie = u16::from_le_bytes([io[0x200], io[0x201]]);
            let if_ = u16::from_le_bytes([io[0x202], io[0x203]]);
            let ime = io[0x208];
            let halted = gba.cpu().is_halted();
            println!(
                "Frame {}: DISPCNT={:04X} IE={:04X} IF={:04X} IME={:02X} halted={}",
                frame, dispcnt, ie, if_, ime, halted
            );
        }
    }
}
