use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..100u32 {
        gba.run_frame_parallel(&mut fb);
    }

    let ewram = gba.mem.wram();
    println!("=== Non-zero EWRAM words in 0x8000-0x8A00 ===");
    for off in (0x8000..0x8A00).step_by(4) {
        let v = u32::from_le_bytes([ewram[off], ewram[off + 1], ewram[off + 2], ewram[off + 3]]);
        if v != 0 {
            println!("  {:08X}: {:08X}", 0x02000000 + off, v);
        }
    }

    // Also check full EWRAM 0x0-0x10000 to see what the decompression
    // IS producing
    println!("\n=== EWRAM non-zero regions (0x0000-0x10000) ===");
    let mut last_nz_end = 0usize;
    for base in (0..0x10000).step_by(0x200) {
        let mut nz = 0;
        for off in (base..base + 0x200).step_by(4) {
            if off + 4 <= ewram.len() {
                let v = u32::from_le_bytes([
                    ewram[off],
                    ewram[off + 1],
                    ewram[off + 2],
                    ewram[off + 3],
                ]);
                if v != 0 {
                    nz += 1;
                }
            }
        }
        if nz > 0 && (base == 0 || base > last_nz_end + 0x200) {
            print!("\n  [{:05X}-{:05X}]: {} nz words", base, base + 0x200, nz);
        } else if nz > 0 {
            print!(" {} nz", nz);
        }
        if nz > 0 {
            last_nz_end = base + 0x200;
        }
    }
    println!();

    // Check what data the decompression at 0x080D0900 is supposed to produce
    // by looking at ROM data at that point
    let rom = gba.mem.rom();
    println!("\n=== ROM around decompression routine ===");
    let decomp_start = 0x080D0900 - 0x08000000;
    for i in 0..8 {
        let off = decomp_start + i * 4;
        let v = u32::from_le_bytes([rom[off], rom[off + 1], rom[off + 2], rom[off + 3]]);
        println!("  {:08X}: {:08X}", 0x080D0900 + i * 4, v);
    }
}
