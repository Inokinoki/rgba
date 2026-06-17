use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut last_pc_in_range = 0u32;
    let mut entries: Vec<(u32, [u32; 16])> = Vec::new();

    for frame in 0..200 {
        for _scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            if pc >= 0x080D0B54 && pc < 0x080D0C10 && last_pc_in_range < 0x080D0B54 {
                let r = gba.cpu().registers();
                if r[1] >= 0x06000000 && r[1] < 0x06018000 && entries.len() < 30 {
                    entries.push((frame, r));
                }
            }
            if pc >= 0x080D0B54 && pc < 0x080D0C10 {
                last_pc_in_range = pc;
            } else {
                last_pc_in_range = 0;
            }
            gba.run_scanline();
        }
    }

    println!("VRAM decompression calls with register dump:");
    for (frame, r) in &entries {
        println!("  Frame {:3}: r0={:08X} r1={:08X} r2={:08X} r3={:08X} r4={:08X} r5={:08X} r6={:08X} r7={:08X} r8={:08X} r9={:08X}", 
                 frame, r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9]);
    }
}
