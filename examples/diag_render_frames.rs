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

fn render_frame(gba: &mut Gba, path: &str) {
    gba.sync_ppu_full();
    gba.sync_ppu();
    let mut pixels = Vec::new();
    let mut sprite_pixels = 0u32;
    let mut unique_colors = std::collections::HashSet::new();
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = gba.get_pixel_tile_mode(x, y);
            unique_colors.insert(c);
            let ppu = gba.ppu();
            if gba.get_sprite_pixel(ppu, x, y).is_some() {
                sprite_pixels += 1;
            }
            let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
            let b = (((c >> 10) & 0x1F) as u32 * 255 / 31) as u8;
            pixels.push((r, g, b));
        }
    }
    write_bmp(path, &pixels, 240, 160).ok();
    let ie = gba.mem().interrupt.ie.bits();
    let if_ = gba.mem().interrupt.if_raw.bits();
    let dc = gba.ppu().get_dispcnt();
    eprintln!(
        "{}: DC=0x{:04X} IE=0x{:04X} IF=0x{:04X} colors={} sprites={} PC=0x{:08X}",
        path,
        dc,
        ie,
        if_,
        unique_colors.len(),
        sprite_pixels,
        gba.cpu_pc()
    );
}

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);

    for i in 0..=1000u32 {
        gba.run_frame();
        match i {
            3 | 4 | 5 | 10 | 20 | 50 | 100 | 200 | 500 | 1000 => {
                render_frame(&mut gba, &format!("/tmp/frame_{:04}.bmp", i));
            }
            _ => {}
        }
    }
}
