use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    let mut last_pc_in_range = 0u32;
    let mut call_count = 0u32;
    let mut last_r1 = 0u32;

    for frame in 0..600 {
        for _scanline in 0..228 {
            let pc = gba.cpu().get_pc();
            if pc >= 0x080D0B54 && pc < 0x080D0C10 && last_pc_in_range < 0x080D0B54 {
                call_count += 1;
                let r = gba.cpu().registers();
                last_r1 = r[1];
            }
            if pc >= 0x080D0B54 && pc < 0x080D0C10 {
                last_pc_in_range = pc;
            } else {
                last_pc_in_range = 0;
            }
            gba.run_scanline();
        }

        if frame % 100 == 99 {
            let bg_writes: usize = gba
                .mem()
                .vram_write_log
                .iter()
                .filter(|(a, _, _)| *a >= 0x06000000 && *a < 0x0600F000)
                .count();
            println!(
                "Frame {}: {} tile loader calls, last r1={:08X}, BG writes={}",
                frame, call_count, last_r1, bg_writes
            );
        }
    }

    println!("\nTotal tile loader calls: {}", call_count);

    // Check VRAM content
    let vram = gba.mem().vram();
    let mut nonzero_tiles = 0;
    for tile in 0..512 {
        let offset = tile * 32;
        if vram[offset..offset + 32].iter().any(|&b| b != 0) {
            nonzero_tiles += 1;
        }
    }
    println!("Nonzero tiles in char block 0: {}/512", nonzero_tiles);
}
