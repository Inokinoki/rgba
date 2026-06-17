use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    let mut last_pc_in_range = 0u32;
    let mut entries: Vec<(u32, [u32; 16])> = Vec::new();

    for frame in 0..200 {
        for _scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            if pc >= 0x080D0B54 && pc < 0x080D0C10 && last_pc_in_range < 0x080D0B54 {
                let r = gba.cpu().registers();
                entries.push((frame, r));
            }
            if pc >= 0x080D0B54 && pc < 0x080D0C10 {
                last_pc_in_range = pc;
            } else {
                last_pc_in_range = 0;
            }
            gba.run_scanline();
        }
    }

    println!("Tile loader function entries:");
    for (frame, r) in &entries {
        let dst_name = if r[1] >= 0x06000000 && r[1] < 0x06018000 {
            "VRAM"
        } else if r[1] >= 0x02000000 {
            "EWRAM"
        } else if r[1] >= 0x03000000 {
            "IWRAM"
        } else {
            "Other"
        };
        println!("  Frame {:3}: pc={:08X} r0={:08X} r1={:08X}({}) r2={:08X} r3={:08X} r4={:08X} r5={:08X} r6={:08X} r7={:08X} r9={:08X}",
                 frame, r[15], r[0], r[1], dst_name, r[2], r[3], r[4], r[5], r[6], r[7], r[9]);
    }
}
