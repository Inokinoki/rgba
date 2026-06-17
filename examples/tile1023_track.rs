use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    gba.mem_mut().vram_log_enabled = true;

    let mut fb = vec![0u32; 240 * 160];
    for frame in 0..240 {
        gba.mem_mut().vram_write_log.clear();
        gba.run_frame_parallel(&mut fb);

        // Check memory VRAM at tile 1023 after each frame
        let mem_vram = gba.mem().vram();
        let tile_1023_offset = 1023 * 32;
        let tile_1023_nonzero: usize = mem_vram[tile_1023_offset..tile_1023_offset + 32]
            .iter()
            .filter(|&&b| b != 0)
            .count();

        if tile_1023_nonzero > 0 || !gba.mem().vram_write_log.is_empty() {
            let ppu_vram = gba.ppu().vram();
            let ppu_tile_1023_nonzero: usize = ppu_vram[tile_1023_offset..tile_1023_offset + 32]
                .iter()
                .filter(|&&b| b != 0)
                .count();

            println!("Frame {:3}: mem tile1023={}/32 nonzero, ppu tile1023={}/32 nonzero, VRAM_writes={}",
                frame, tile_1023_nonzero, ppu_tile_1023_nonzero, gba.mem().vram_write_log.len());

            if tile_1023_nonzero > 0 && tile_1023_nonzero < 32 {
                print!("  MEM: ");
                for i in 0..32 {
                    print!("{:02X} ", mem_vram[tile_1023_offset + i]);
                }
                println!();
            }
        }
    }
}
