use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Check VRAM state before frame
    let vram = gba.mem.vram();
    let mut nonzero_before = 0;
    for i in 0..0x8000 {
        if vram[i] != 0 {
            nonzero_before += 1;
        }
    }
    println!(
        "Before any frames: {} non-zero bytes in VRAM",
        nonzero_before
    );

    // Run 1 frame
    gba.run_frame_parallel(&mut fb);
    let vram = gba.mem.vram();
    let mut nonzero_after1 = 0;
    for i in 0..0x8000 {
        if vram[i] != 0 {
            nonzero_after1 += 1;
        }
    }
    println!(
        "After 1 frame: {} non-zero bytes in VRAM (delta={})",
        nonzero_after1,
        nonzero_after1 as i64 - nonzero_before as i64
    );

    // Run to frame 200
    for _ in 1..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Dump VRAM tile map for BG0 (tile base = 0xC000)
    let vram = gba.mem.vram();
    println!("\nBG0 tile map at frame 200 (tb=0xC000):");
    for row in 0..4 {
        let mut line = String::new();
        for col in 0..32 {
            let off = 0xC000 + (row * 64 + col) * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            if entry != 0 {
                line.push_str(&format!(
                    " [{:03}:{:X}]",
                    entry & 0x3FF,
                    (entry >> 12) & 0xF
                ));
            } else {
                line.push_str(" .");
            }
        }
        if line.contains('[') {
            println!("  Row {}:{}", row, line);
        }
    }

    // Check VRAM differences between frame 200 and 201
    let vram_before: Vec<u8> = vram.to_vec();
    gba.run_frame_parallel(&mut fb);
    let vram_after = gba.mem.vram();

    let mut diff_count = 0;
    let mut first_diffs = Vec::new();
    for i in 0..0x18000 {
        if vram_before[i] != vram_after[i] {
            diff_count += 1;
            if first_diffs.len() < 20 {
                first_diffs.push((i, vram_before[i], vram_after[i]));
            }
        }
    }
    println!("\nVRAM diff frame 200->201: {} bytes changed", diff_count);
    for (off, before, after) in &first_diffs {
        println!("  0x{:05X}: {:02X}->{:02X}", off, before, after);
    }
}
