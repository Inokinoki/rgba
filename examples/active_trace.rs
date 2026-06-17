use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for scanline in 0..228 {
            let halted_before = gba.cpu().is_halted();
            gba.run_scanline();
            let halted_after = gba.cpu().is_halted();

            if !halted_after && frame >= 190 && frame <= 192 {
                let pc = gba.cpu().get_pc();
                let r = gba.cpu().registers();
                println!(
                    "Frame {} SL {:3}: ACTIVE pc={:08X} r0={:08X} r1={:08X} lr={:08X}",
                    frame, scanline, pc, r[0], r[1], r[14]
                );
            }
        }
    }
}
