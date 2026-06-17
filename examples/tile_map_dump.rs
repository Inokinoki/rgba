use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let vram = gba.mem.vram().to_vec();

    // For each BG, check what tile is at screen position (0,0) to (8,8)
    for bg in 0..4u32 {
        let bgcnt = gba.mem.read_half(0x04000008 + bg * 2);
        let pri = bgcnt & 3;
        let tb = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let size = (bgcnt >> 14) & 3;
        let hscroll = (gba.mem.read_half(0x04000010 + bg * 4) & 0x1FF) as usize;
        let vscroll = (gba.mem.read_half(0x04000012 + bg * 4) & 0x1FF) as usize;

        // Check tile dimensions based on size
        let (map_w, map_h) = match size {
            0 => (32, 32),
            1 => (64, 32),
            2 => (32, 64),
            3 => (64, 64),
            _ => (32, 32),
        };

        println!(
            "BG{}: pri={} tb=0x{:X} size={} map={}x{} scroll=({},{})",
            bg, pri, tb, size, map_w, map_h, hscroll, vscroll
        );

        // Check visible tiles in a 4x4 grid starting at (0,0)
        for y in 0..4 {
            let mut line = String::new();
            for x in 0..30 {
                let map_x = (x * 8 + hscroll) % (map_w * 8);
                let map_y = (y * 8 + vscroll) % (map_h * 8);
                let tile_col = map_x / 8;
                let tile_row = map_y / 8;

                // Handle screen base blocks for 64-wide maps
                let screen_base = if map_w == 64 {
                    let block_col = tile_col / 32;
                    let block_row = tile_row / 32;
                    tb + (block_row * 2 + block_col) * 0x800
                } else {
                    tb
                };
                let local_col = tile_col % 32;
                let local_row = tile_row % 32;

                let entry_off = screen_base + (local_row * 32 + local_col) * 2;
                let entry = if entry_off + 1 < vram.len() {
                    u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]])
                } else {
                    0
                };

                let tile = entry & 0x3FF;
                let pal = (entry >> 12) & 0xF;
                if tile != 0x3FF {
                    line.push_str(&format!(" T{}:{}", tile, pal));
                } else {
                    line.push_str(" .");
                }
            }
            if line.contains('T') {
                println!("  y={}: {}", y, line);
            }
        }
    }
}
