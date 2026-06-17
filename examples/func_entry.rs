use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut func_entries: Vec<[u32; 16]> = Vec::new();
    let target_start = 0x080D0B54u32;
    let target_end = 0x080D0C10u32;

    for _frame in 0..10 {
        for _scanline in 0..228 {
            let ipc = gba.cpu().get_instruction_pc();
            if ipc >= target_start && ipc < target_end && func_entries.is_empty() {
                // Check if this looks like a function entry
                if ipc == 0x080D0B54 {
                    func_entries.push(gba.cpu().registers());
                }
            }
            gba.run_scanline();
        }
    }

    println!("Function entries captured: {}", func_entries.len());
    for (i, r) in func_entries.iter().enumerate() {
        println!("Entry {}: r0={:08X} r1={:08X} r2={:08X} r3={:08X} r4={:08X} r5={:08X} r6={:08X} r7={:08X} r8={:08X} r9={:08X}", 
                 i, r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7], r[8], r[9]);
    }
}
