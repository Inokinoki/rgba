use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;

    let mut fb = vec![0u32; 240 * 160];
    for frame in 0..192 {
        gba.mem_mut().vram_write_log.clear();
        gba.run_frame_parallel(&mut fb);

        if frame >= 190 {
            // Check writes to tile 1023 area (0x7FE0-0x7FFF in memory VRAM)
            let tile_1023_writes: Vec<_> = gba
                .mem()
                .vram_write_log
                .iter()
                .filter(|&&(addr, _, _)| {
                    let off = (addr & 0x0FFFFFFF) - 0x06000000;
                    off >= 0x7FE0 && off <= 0x7FFF
                })
                .cloned()
                .collect();

            if !tile_1023_writes.is_empty() {
                println!(
                    "Frame {}: {} writes to tile 1023 area",
                    frame,
                    tile_1023_writes.len()
                );
                for &(addr, pc, val) in &tile_1023_writes {
                    let off = (addr & 0x0FFFFFFF) - 0x06000000;
                    println!("  offset={:05X} val={:02X} pc={:08X}", off, val, pc);
                }

                // Now check what's actually in memory at those offsets
                let vram = gba.mem().vram();
                println!("  Memory at 0x7FE0:");
                for i in 0..32 {
                    print!("{:02X} ", vram[0x7FE0 + i]);
                }
                println!();
            }
        }
    }
}
