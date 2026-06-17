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
    let test_roms = [
        (
            "/home/ubuntu/Builds/gba-tests/ppu/shades.gba",
            "/tmp/tv_shades",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/ppu/stripes.gba",
            "/tmp/tv_stripes",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/ppu/hello.gba",
            "/tmp/tv_hello",
        ),
    ];

    for (rom_path, prefix) in &test_roms {
        let mut gba = Gba::new();
        if gba.load_rom_path(rom_path).is_err() {
            continue;
        }
        for _ in 0..500 {
            gba.run_frame();
        }
        gba.sync_ppu_full();
        gba.sync_ppu();

        let dc = gba.ppu().get_dispcnt();
        println!("{}: DISPCNT={:#06X} mode={}", prefix, dc, dc & 0x7);

        let mut pixels = Vec::new();
        for y in 0..160u16 {
            for x in 0..240u16 {
                let c = gba.get_pixel_tile_mode(x, y);
                let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
                let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
                let b = ((c >> 10) as u32) as u8 * 255 / 31;
                pixels.push((r, g, b));
            }
        }
        let bmp_path = format!("{}.bmp", prefix);
        let png_path = format!("{}.png", prefix);
        write_bmp(&bmp_path, &pixels, 240, 160).ok();
        std::process::Command::new("convert")
            .arg(&bmp_path)
            .arg(&png_path)
            .output()
            .ok();
        println!("  -> {}", png_path);
    }
}
