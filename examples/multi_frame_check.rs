use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];
    let mut frame_num = 0u32;

    for target in [60u32, 120, 180, 240, 300, 600, 1200, 1800] {
        while frame_num < target {
            gba.run_frame_parallel(&mut framebuffer);
            frame_num += 1;
        }

        gba.sync_ppu_full();
        gba.sync_ppu();
        let ppu = gba.ppu();
        let dispcnt = ppu.get_dispcnt();
        let mode = dispcnt & 0x7;
        let bg_en = (dispcnt >> 8) & 0xF;
        let obj_en = (dispcnt >> 12) & 1;

        let vram = ppu.vram();
        let mut tiles_loaded = 0u32;
        for t in 0..1024u32 {
            let start = t as usize * 32;
            if start + 32 > vram.len() { break; }
            let mut has = false;
            for b in 0..32 {
                if vram[start + b] != 0 { has = true; break; }
            }
            if has { tiles_loaded += 1; }
        }

        // Count unique pixel colors in framebuffer
        let mut colors = std::collections::HashMap::new();
        for &p in &framebuffer {
            *colors.entry(p).or_insert(0u32) += 1;
        }
        let mut sorted: Vec<_> = colors.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        println!("\nFrame {}: mode={} BG_en={:#04X} OBJ_en={} tiles={}", 
                 target, mode, bg_en, obj_en, tiles_loaded);
        println!("  Top 5 colors: {:?}", sorted.iter().take(5).map(|(&c, &n)| (format!("{:#010X}", c), n)).collect::<Vec<_>>());
        
        for bg in 0..4 {
            if (bg_en >> bg) & 1 == 0 { continue; }
            let bgcnt = ppu.get_bgcnt(bg);
            let pri = bgcnt & 0x3;
            let tile_base = ((bgcnt >> 2) & 0x3) * 0x4000;
            let scr_base = ((bgcnt >> 8) & 0x1F) * 0x800;
            let is_8bpp = (bgcnt & 0x80) != 0;
            println!("  BG{}: pri={} tile_base={:#06X} scr_base={:#06X} {}bpp", 
                     bg, pri, tile_base, scr_base, if is_8bpp { "8" } else { "4" });
        }
    }
}
