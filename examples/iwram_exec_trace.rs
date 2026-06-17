use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for scanline in 0..228 {
            gba.run_scanline();

            if frame == 191 && scanline >= 164 && scanline <= 175 {
                let pc = gba.cpu().get_pc();
                let mode = gba.cpu().get_mode();
                let thumb = gba.cpu().is_thumb_mode();
                let r = gba.cpu().registers();
                let halted = gba.cpu().is_halted();
                if !halted {
                    println!("F191 SL{:3} PC={:08X} mode={:?} thumb={} r0={:08X} r1={:08X} r14={:08X} CPSR={:08X}",
                             scanline, pc, mode, thumb, r[0], r[1], r[14], gba.cpu().get_cpsr());
                }
            }
        }
    }
}
