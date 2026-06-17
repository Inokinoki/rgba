use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut last_pc_in_range = 0u32;
    let mut entry_regs: Vec<[u32; 16]> = Vec::new();

    for _frame in 0..10 {
        for _scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            // Check if PC just entered the range 0x080D0B54-0x080D0C10
            if pc >= 0x080D0B54 && pc < 0x080D0C10 && last_pc_in_range < 0x080D0B54 {
                if entry_regs.len() < 10 {
                    entry_regs.push(gba.cpu().registers());
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

    println!("Entries into tile loader: {}", entry_regs.len());
    for (i, r) in entry_regs.iter().enumerate() {
        println!("Entry {}: pc={:08X} r0={:08X} r1={:08X} r2={:08X} r3={:08X} r4={:08X} r5={:08X} r6={:08X} r7={:08X}", 
                 i, r[15], r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]);
    }
}
