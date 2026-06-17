use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            if pc >= 0x08000000 && pc < 0x080D0000 && frame >= 190 && scanline < 10 {
                // CPU is executing code in main ROM area after frame 190
                // Log what instruction it's executing
                if scanline == 0 && frame <= 195 {
                    let r = gba.cpu().registers();
                    println!("Frame {} SL {} PC={:08X} r0={:08X} r1={:08X} r2={:08X} r3={:08X} r4={:08X}", 
                             frame, scanline, pc, r[0], r[1], r[2], r[3], r[4]);
                }
            }
            gba.run_scanline();
        }
    }
}
