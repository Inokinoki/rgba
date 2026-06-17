use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut last_pc_in_func = 0u32;
    let mut in_func = false;
    let mut call_log: Vec<(u32, u32, u32)> = Vec::new(); // (frame, src, dst)

    for frame in 0..200 {
        for _scanline in 0..228 {
            let pc = gba.cpu().get_pc();

            // Detect entry into decompression function area
            if pc >= 0x080D0900 && pc < 0x080D0D00 {
                if !in_func {
                    // Just entered the function area
                    let r = gba.cpu().registers();
                    let dst = r[1];
                    let src = r[0];
                    if call_log.len() < 100 {
                        call_log.push((frame, src, dst));
                    }
                }
                in_func = true;
                last_pc_in_func = pc;
            } else {
                in_func = false;
            }

            gba.run_scanline();
        }
    }

    println!("Decompression function calls (first 100):");
    let mut vram_calls = 0;
    let mut ewram_calls = 0;
    for (i, (frame, src, dst)) in call_log.iter().enumerate() {
        let dst_type = if *dst >= 0x06000000 && *dst < 0x06018000 {
            "VRAM"
        } else if *dst >= 0x02000000 {
            "EWRAM"
        } else if *dst >= 0x03000000 {
            "IWRAM"
        } else {
            "Other"
        };
        if dst_type == "VRAM" {
            vram_calls += 1;
        } else {
            ewram_calls += 1;
        }
        println!(
            "  {:3}: frame={:3} src={:08X} dst={:08X} ({})",
            i, frame, src, dst, dst_type
        );
    }
    println!("\nTotal: {} VRAM, {} EWRAM/IWRAM", vram_calls, ewram_calls);
}
