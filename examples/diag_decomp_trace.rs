use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // We need to trace the decompression more carefully.
    // The log fills up at 100K entries over 240 frames.
    // Let's run frame by frame and check tile content each frame.

    for frame in 0..30 {
        gba.run_frame_parallel(&mut fb);
        gba.sync_ppu_full();
        let vram = gba.mem.vram();

        let mut nonzero_0343 = 0;
        let mut nonzero_344_472 = 0;
        let mut nonzero_473_622 = 0;

        for tid in 0..=343u32 {
            let off = tid as usize * 32;
            if vram[off..off + 32].iter().any(|&b| b != 0) {
                nonzero_0343 += 1;
            }
        }
        for tid in 344..=472u32 {
            let off = tid as usize * 32;
            if vram[off..off + 32].iter().any(|&b| b != 0) {
                nonzero_344_472 += 1;
            }
        }
        for tid in 473..=622u32 {
            let off = tid as usize * 32;
            if vram[off..off + 32].iter().any(|&b| b != 0) {
                nonzero_473_622 += 1;
            }
        }

        eprintln!(
            "Frame {:3}: tiles 0-343: {} | 344-472: {} | 473-622: {}",
            frame, nonzero_0343, nonzero_344_472, nonzero_473_622
        );
    }

    // Now let's look at the decompression ROM code at 0x080D0BFA
    // This is THUMB mode code. Let's dump the bytes.
    let rom = gba.mem.rom();
    let base = 0x080D0000 - 0x08000000;
    eprintln!("\nROM at 0x080D0BEC-0x080D0C10 (THUMB):");
    for off in (0..0x30).step_by(2) {
        let addr = base + 0xBEC + off;
        if addr + 1 < rom.len() {
            let hw = u16::from_le_bytes([rom[addr], rom[addr + 1]]);
            eprintln!("  0x{:08X}: 0x{:04X}", 0x080D0BEC + off, hw);
        }
    }

    // Let's also look at the broader decompression function
    eprintln!("\nROM at 0x080D0B80-0x080D0CA0 (THUMB):");
    for off in (0..0x120).step_by(2) {
        let addr = base + 0xB80 + off;
        if addr + 1 < rom.len() {
            let hw = u16::from_le_bytes([rom[addr], rom[addr + 1]]);
            eprintln!("  0x{:08X}: 0x{:04X}", 0x080D0B80 + off, hw);
        }
    }
}
