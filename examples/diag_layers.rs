use rgba::Gba;
use std::io::Write;

fn write_bmp(path: &str, pixels: &[(u8, u8, u8)], width: u32, height: u32) -> std::io::Result<()> {
    let row_size = (width * 3 + 3) & !3;
    let pixel_data_size = row_size * height;
    let file_size = 54 + pixel_data_size;
    let mut f = std::fs::File::create(path)?;
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
    f.write_all(&[0u8; 4])?;
    f.write_all(&54u32.to_le_bytes())?;
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(width as i32).to_le_bytes())?;
    f.write_all(&(height as i32).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&24u16.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&pixel_data_size.to_le_bytes())?;
    f.write_all(&2835u32.to_le_bytes())?;
    f.write_all(&2835u32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    for y in (0..height).rev() {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let (r, g, b) = pixels[idx];
            f.write_all(&[b, g, r])?;
        }
        for _ in 0..(row_size - width * 3) {
            f.write_all(&[0u8])?;
        }
    }
    Ok(())
}

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);

    let target_frame = 1000u32;
    for _ in 0..target_frame {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    // Print PPU state
    {
        let ppu = gba.ppu();
        eprintln!(
            "Frame {}: DC=0x{:04X} mode={}",
            target_frame,
            ppu.get_dispcnt(),
            ppu.get_display_mode()
        );
        for bg in 0..4 {
            let bgcnt = ppu.get_bgcnt(bg);
            let prio = ppu.get_bg_priority(bg);
            eprintln!(
                "  BG{}: bgcnt=0x{:04X} prio={} enabled={} hofs={} vofs={}",
                bg,
                bgcnt,
                prio,
                ppu.is_bg_enabled(bg),
                ppu.get_bg_hofs(bg),
                ppu.get_bg_vofs(bg)
            );
        }
    }

    // Render composite
    {
        let mut pixels = Vec::new();
        for y in 0..160u16 {
            for x in 0..240u16 {
                let c = gba.get_pixel_tile_mode(x, y);
                let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
                let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
                let b = (((c >> 10) & 0x1F) as u32 * 255 / 31) as u8;
                pixels.push((r, g, b));
            }
        }
        write_bmp("/tmp/composite.bmp", &pixels, 240, 160).ok();
    }

    // Render each BG layer
    for layer in 0..4 {
        let ppu = gba.ppu();
        let mode = ppu.get_display_mode();
        let mut pixels = Vec::new();
        for y in 0..160u16 {
            for x in 0..240u16 {
                let c = if ppu.is_bg_enabled(layer) {
                    match gba.get_bg_pixel(ppu, mode, layer, x, y) {
                        Some(c) => c,
                        None => 0x7FFF,
                    }
                } else {
                    0
                };
                let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
                let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
                let b = (((c >> 10) & 0x1F) as u32 * 255 / 31) as u8;
                pixels.push((r, g, b));
            }
        }
        write_bmp(&format!("/tmp/layer_bg{}.bmp", layer), &pixels, 240, 160).ok();
    }

    // Render OBJ layer
    {
        let mut pixels = Vec::new();
        for y in 0..160u16 {
            for x in 0..240u16 {
                let ppu = gba.ppu();
                let c = match gba.get_sprite_pixel(ppu, x, y) {
                    Some((c, _)) => c,
                    None => 0x7FFF,
                };
                let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
                let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
                let b = (((c >> 10) & 0x1F) as u32 * 255 / 31) as u8;
                pixels.push((r, g, b));
            }
        }
        write_bmp("/tmp/layer_obj.bmp", &pixels, 240, 160).ok();
    }

    // Check BG screen maps
    {
        let ppu = gba.ppu();
        let vram = ppu.vram();
        for bg in 0..4 {
            let bgcnt = ppu.get_bgcnt(bg);
            let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
            let tile_base = (bgcnt & 0x3) as usize * 0x4000;
            let prio = ppu.get_bg_priority(bg);

            let mut nonzero_tiles = 0;
            let mut unique_tiles = std::collections::HashSet::new();
            for i in 0..1024 {
                let off = map_base + i * 2;
                if off + 1 < vram.len() {
                    let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
                    let tile = entry & 0x3FF;
                    if tile != 0 {
                        nonzero_tiles += 1;
                        unique_tiles.insert(tile);
                    }
                }
            }
            eprintln!(
                "BG{} (prio={}): map=0x{:05X} tile=0x{:05X} nonzero={}/1024 unique_tiles={}",
                bg,
                prio,
                map_base,
                tile_base,
                nonzero_tiles,
                unique_tiles.len()
            );
        }
    }

    eprintln!("Done. Files: /tmp/composite.bmp /tmp/layer_bg*.bmp /tmp/layer_obj.bmp");
}
