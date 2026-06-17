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

fn render(gba: &mut Gba, path: &str) {
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
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    let checkpoints = [
        (100, "/tmp/hm_long_f100.bmp"),
        (500, "/tmp/hm_long_f500.bmp"),
        (1000, "/tmp/hm_long_f1000.bmp"),
        (2000, "/tmp/hm_long_f2000.bmp"),
        (5000, "/tmp/hm_long_f5000.bmp"),
        (10000, "/tmp/hm_long_f10000.bmp"),
    ];

    let mut frame = 0u32;
    let mut next_cp = 0;

    while next_cp < checkpoints.len() {
        let target = checkpoints[next_cp].0;
        while frame < target {
            gba.run_frame();
            frame += 1;
        }
        let path = checkpoints[next_cp].1;
        render(&mut gba, path);
        let dc = gba.ppu().get_dispcnt();
        println!(
            "Frame {}: DC={:#06X} PC={:#010X}",
            frame,
            dc,
            gba.cpu().get_instruction_pc()
        );
        next_cp += 1;
    }

    for (_, bmp) in &checkpoints {
        let png = bmp.replace(".bmp", ".png");
        std::process::Command::new("convert")
            .arg(bmp)
            .arg(&png)
            .output()
            .ok();
    }
    println!("Done.");
}
