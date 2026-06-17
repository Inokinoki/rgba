use rgba::Gba;

fn save_ppm(fb: &[u32], path: &str) {
    let width = 240;
    let height = 160;
    let mut out = Vec::with_capacity(width * height * 3 + 100);
    out.extend_from_slice(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    for y in 0..height {
        for x in 0..width {
            let pixel = fb[y * width + x];
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            out.extend_from_slice(&[r, g, b]);
        }
    }
    std::fs::write(path, &out).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();

    let ppu = &gba.ppu;
    let mem = &gba.mem;
    let vram = ppu.vram();

    // Render each BG layer separately
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let screen_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let char_base = ((bgcnt >> 2) & 3) as usize * 0x4000;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let bg_size = (bgcnt >> 14) & 3;
        let is_8bpp = (bgcnt & 0x80) != 0;
        let width = match bg_size {
            0 => 256u16,
            1 => 512,
            2 => 256,
            3 => 512,
            _ => 256,
        };
        let height = match bg_size {
            0 => 256u16,
            1 => 256,
            2 => 512,
            3 => 512,
            _ => 256,
        };
        let tile_w = width / 8;
        let tile_h = height / 8;

        eprintln!(
            "BG{}: bgcnt=0x{:04X} cb=0x{:05X} sb=0x{:05X} hofs={} vofs={} size={} 8bpp={}",
            bg, bgcnt, char_base, screen_base, hofs, vofs, bg_size, is_8bpp
        );

        let mut layer_fb = vec![0u32; 240 * 160];
        for y in 0..160u16 {
            for x in 0..240u16 {
                let bg_x = ((x as u32 + hofs as u32) % width as u32) as u16;
                let bg_y = ((y as u32 + vofs as u32) % height as u32) as u16;
                let tile_x = bg_x / 8;
                let tile_y = bg_y / 8;
                let pixel_x = (bg_x % 8) as u8;
                let pixel_y = (bg_y % 8) as u8;

                let entry =
                    ppu.get_screen_entry(screen_base, tile_x, tile_y, bg_size, tile_w, tile_h);
                let (tile_num, flip_h, flip_v, palette_num, _) =
                    rgba::Ppu::parse_screen_entry(entry);

                let px = if flip_h { 7 - pixel_x } else { pixel_x };
                let py = if flip_v { 7 - pixel_y } else { pixel_y };

                let tile_offset = char_base + tile_num as usize * 32;
                let row_offset = tile_offset + py as usize * 4;

                let color_index = if is_8bpp {
                    vram.get(tile_offset + py as usize * 8 + px as usize)
                        .copied()
                        .unwrap_or(0)
                } else {
                    let byte = vram
                        .get(row_offset + (px as usize / 2))
                        .copied()
                        .unwrap_or(0);
                    if px % 2 == 0 {
                        byte & 0x0F
                    } else {
                        byte >> 4
                    }
                };

                if color_index != 0 {
                    let pal_idx = if is_8bpp {
                        color_index as u16
                    } else {
                        palette_num * 16 + color_index as u16
                    };
                    let c = mem.read_palette_color(0, pal_idx);
                    let r = ((c & 0x1F) as u32 * 255 / 31) << 16;
                    let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) << 8;
                    let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
                    layer_fb[y as usize * 240 + x as usize] = r | g | b;
                }
            }
        }

        let nonzero: usize = layer_fb.iter().filter(|&&p| p != 0).count();
        eprintln!("  Non-zero pixels: {}/38400", nonzero);
        save_ppm(&layer_fb, &format!("/tmp/bg{}_layer.ppm", bg));
    }

    // Also dump tile 394 raw data
    let bg0cnt = ppu.get_bgcnt(0);
    let char_base = ((bg0cnt >> 2) & 3) as usize * 0x4000;
    for tile_id in [394, 317, 318, 473] {
        let offset = char_base + tile_id * 32;
        if offset + 32 <= vram.len() {
            let hex: String = (0..32)
                .map(|b| format!("{:02X}", vram[offset + b]))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("Tile {} @0x{:05X}: {}", tile_id, offset, hex);
        }
    }

    eprintln!("Done. Convert PPMs to PNG:");
    for bg in 0..4 {
        let _ = std::process::Command::new("convert")
            .args([
                &format!("/tmp/bg{}_layer.ppm", bg),
                &format!("/tmp/bg{}_layer.png", bg),
            ])
            .status();
    }
}
