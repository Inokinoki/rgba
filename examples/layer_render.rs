use rgba::Gba;
use rgba::KeyState;

fn save_bmp(fb: &[u32], path: &str) {
    let width = 240u32;
    let height = 160u32;
    let row_size = (width * 3 + 3) & !3;
    let pixel_size = row_size * height;
    let file_size = 54 + pixel_size;
    let mut out = Vec::with_capacity(file_size as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&[1, 0, 24, 0]);
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&pixel_size.to_le_bytes());
    out.extend_from_slice(&[0; 16]);
    out.extend_from_slice(&[0; 16]);
    for y in (0..height).rev() {
        let row_start = (y * width) as usize;
        let mut row = Vec::with_capacity(row_size as usize);
        for x in 0..width {
            let pixel = fb[row_start + x as usize];
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            row.extend_from_slice(&[b, g, r]);
        }
        while row.len() < row_size as usize {
            row.push(0);
        }
        out.extend_from_slice(&row);
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
    gba.input.press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input.release_key(KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut fb);
    }
    for _ in 0..80 {
        gba.input.press_key(KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut fb);
        }
        gba.input.release_key(KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut fb);
        }
    }

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let palette = gba.mem().palette();
    let vram = ppu.vram();

    for bg in 0..4 {
        let mut layer_fb = vec![0u32; 240 * 160];
        let bgcnt = ppu.get_bgcnt(bg);
        let tile_base = ppu.get_bg_tile_base(bg) as usize;
        let map_base = ppu.get_bg_map_base(bg) as usize;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);

        for y in 0..160u16 {
            for x in 0..240u16 {
                let bg_x = ((x as u32 + hofs as u32) % 256) as u16;
                let bg_y = ((y as u32 + vofs as u32) % 256) as u16;
                let tile_x = bg_x / 8;
                let tile_y = bg_y / 8;
                let pixel_x = bg_x % 8;
                let pixel_y = bg_y % 8;

                let entry = ppu.get_screen_entry(map_base, tile_x, tile_y, 0, 32, 32);
                let tile_num = entry & 0x3FF;
                let flip_h = (entry & 0x400) != 0;
                let flip_v = (entry & 0x800) != 0;
                let palette_num = (entry >> 12) & 0xF;

                let px = if flip_h { 7 - pixel_x } else { pixel_x };
                let py = if flip_v { 7 - pixel_y } else { pixel_y };

                let tile_offset = tile_base + (tile_num as usize * 32);
                if tile_offset + 32 > vram.len() {
                    continue;
                }
                let row_off = tile_offset + (py as usize * 4);
                let byte_val = vram[row_off + (px as usize / 2)];
                let color_index = if px % 2 == 0 {
                    byte_val & 0x0F
                } else {
                    byte_val >> 4
                };

                if color_index == 0 {
                    continue;
                }

                let pal_index = (palette_num as usize * 16) + color_index as usize;
                let pal_offset = pal_index * 2;
                if pal_offset + 1 < palette.len() {
                    let color = u16::from_le_bytes([palette[pal_offset], palette[pal_offset + 1]]);
                    let r = ((color & 0x1F) as u32 * 255 / 31) << 16;
                    let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
                    let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                    layer_fb[(y as usize) * 240 + (x as usize)] = r | g | b;
                }
            }
        }

        let path = format!("/tmp/bg{}_layer.bmp", bg);
        save_bmp(&layer_fb, &path);
        println!(
            "BG{} layer saved (cnt={:#06X}, tile={:#X}, map={:#X}, hofs={}, vofs={})",
            bg, bgcnt, tile_base, map_base, hofs, vofs
        );
    }

    for bg in 0..4 {
        let bmp = format!("/tmp/bg{}_layer.bmp", bg);
        let png = format!("/tmp/bg{}_layer.png", bg);
        std::process::Command::new("convert")
            .args([&bmp, &png])
            .status()
            .expect("convert failed");
    }
    println!("All layers converted to PNG");
}
