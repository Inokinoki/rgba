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

fn render_test_rom(gba: &mut Gba, path: &str) {
    gba.sync_ppu_full();
    gba.sync_ppu();
    let mut pixels = Vec::new();
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = gba.get_pixel_tile_mode(x, y);
            let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
            let b = ((c >> 10) as u32 * 255 / 31) as u8;
            pixels.push((r, g, b));
        }
    }
    write_bmp(path, &pixels, 240, 160).ok();
}

fn main() {
    let test_roms = [
        (
            "/home/ubuntu/Builds/gba-tests/arm/arm.gba",
            "/tmp/test_vis_arm.png",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/thumb/thumb.gba",
            "/tmp/test_vis_thumb.png",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/ppu/shades/shades.gba",
            "/tmp/test_vis_shades.png",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/ppu/stripes/stripes.gba",
            "/tmp/test_vis_stripes.png",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/ppu/hello/hello.gba",
            "/tmp/test_vis_hello.png",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/bios/bios.gba",
            "/tmp/test_vis_bios.png",
        ),
    ];

    for (rom_path, png_path) in &test_roms {
        let mut gba = Gba::new();
        if gba.load_rom_path(rom_path).is_err() {
            continue;
        }
        for _ in 0..500 {
            gba.run_frame();
        }
        let bmp_path = png_path.replace(".png", ".bmp");
        render_test_rom(&mut gba, &bmp_path);
        std::process::Command::new("convert")
            .arg(&bmp_path)
            .arg(png_path)
            .output()
            .ok();
        println!("Rendered: {} -> {}", rom_path, png_path);
    }
}
