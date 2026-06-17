use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    let mut iwram_exec_count = 0u32;

    for frame in 0..200 {
        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let blank = (dispcnt >> 7) & 1;

        for scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            if pc >= 0x0300_0000 && pc < 0x0400_0000 {
                iwram_exec_count += 1;
            }
            gba.run_scanline();
        }
    }

    println!("IWRAM instruction execution count: {}", iwram_exec_count);
}
