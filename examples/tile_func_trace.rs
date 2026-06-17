use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    let mut func_call_count = 0u32;
    let mut func_params: Vec<[u32; 8]> = Vec::new();

    for frame in 0..300 {
        for scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            if pc == 0x080D0B55 || pc == 0x080D0B54 || pc == 0x080D0B56 {
                func_call_count += 1;
                if func_params.len() < 20 {
                    let r = gba.cpu().registers();
                    func_params.push([r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]]);
                }
            }
            gba.run_scanline();
        }

        if frame % 100 == 99 {
            println!(
                "Frame {}: func call count = {}, BG writes = {}",
                frame,
                func_call_count,
                gba.mem()
                    .vram_write_log
                    .iter()
                    .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600F000)
                    .count()
            );
        }
    }

    println!("\nFunction call count: {}", func_call_count);
    println!("\nRegister snapshots at function entry:");
    for (i, r) in func_params.iter().enumerate() {
        println!("  Call {}: r0={:08X} r1={:08X} r2={:08X} r3={:08X} r4={:08X} r5={:08X} r6={:08X} r7={:08X}", 
                 i, r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]);
    }
}
