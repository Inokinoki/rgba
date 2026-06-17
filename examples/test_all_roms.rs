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

fn get_pixel_mode4(gba: &Gba, x: u16, y: u16) -> (u8, u8, u8) {
    let vram = gba.ppu().vram();
    let pal = gba.mem().palette();
    let page: usize = if gba.ppu().get_dispcnt() & 0x10 != 0 {
        0xA000
    } else {
        0
    };
    let offset = page + (y as usize * 240 + x as usize);
    if offset >= vram.len() {
        return (0, 0, 0);
    }
    let index = vram[offset] as usize;
    let pal_offset = index * 2;
    if pal_offset + 1 >= pal.len() {
        return (0, 0, 0);
    }
    let color = u16::from_le_bytes([pal[pal_offset], pal[pal_offset + 1]]);
    let r = ((color & 0x1F) as u32 * 255 / 31) as u8;
    let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) as u8;
    let b = (((color >> 10) & 0x1F) as u32 * 255 / 31) as u8;
    (r, g, b)
}

fn main() {
    let tests = [
        ("arm", "/home/ubuntu/Builds/gba-tests/arm/arm.gba"),
        ("thumb", "/home/ubuntu/Builds/gba-tests/thumb/thumb.gba"),
        ("bios", "/home/ubuntu/Builds/gba-tests/bios/bios.gba"),
        ("unsafe", "/home/ubuntu/Builds/gba-tests/unsafe/unsafe.gba"),
        ("memory", "/home/ubuntu/Builds/gba-tests/memory/memory.gba"),
        ("nes", "/home/ubuntu/Builds/gba-tests/nes/nes.gba"),
        ("hello", "/home/ubuntu/Builds/gba-tests/ppu/hello.gba"),
        ("shades", "/home/ubuntu/Builds/gba-tests/ppu/shades.gba"),
        ("stripes", "/home/ubuntu/Builds/gba-tests/ppu/stripes.gba"),
        (
            "flash128",
            "/home/ubuntu/Builds/gba-tests/save/flash128.gba",
        ),
        ("flash64", "/home/ubuntu/Builds/gba-tests/save/flash64.gba"),
        ("save_none", "/home/ubuntu/Builds/gba-tests/save/none.gba"),
        ("sram", "/home/ubuntu/Builds/gba-tests/save/sram.gba"),
    ];

    let mut png_paths = vec![];

    for (name, path) in &tests {
        if !std::path::Path::new(path).exists() {
            println!("{}: file not found, skipping", name);
            continue;
        }
        let mut gba = Gba::new();
        gba.load_rom_path(path).unwrap();

        if *name == "flash128" {
            gba.mem_mut().set_save_type(rgba::SaveType::Flash128K);
        } else if *name == "flash64" {
            gba.mem_mut().set_save_type(rgba::SaveType::Flash64K);
        } else if *name == "sram" {
            gba.mem_mut().set_save_type(rgba::SaveType::Sram);
        }

        for _ in 0..300 {
            gba.run_frame();
        }

        gba.sync_ppu_full();
        gba.sync_ppu();

        let dc = gba.ppu().get_dispcnt();
        let mode = dc & 7;
        let r12 = gba.cpu().get_reg(12);
        let pc = gba.cpu().get_instruction_pc();
        println!(
            "{:12}: DC={:#06X} mode={} PC={:#010X} R12={:#010X}",
            name, dc, mode, pc, r12
        );

        let mut pixels = Vec::new();
        let mut non_zero = 0u32;
        for y in 0..160u16 {
            for x in 0..240u16 {
                let (r, g, b) = if mode == 3 {
                    let vram = gba.ppu().vram();
                    let off = (y as usize * 240 + x as usize) * 2;
                    if off + 1 < vram.len() {
                        let c = u16::from_le_bytes([vram[off], vram[off + 1]]);
                        let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
                        let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
                        let b = (((c >> 10) as u32) * 255 / 31) as u8;
                        (r, g, b)
                    } else {
                        (0, 0, 0)
                    }
                } else if mode == 4 {
                    get_pixel_mode4(&gba, x, y)
                } else {
                    let c = gba.get_pixel_tile_mode(x, y);
                    let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
                    let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
                    let b = (((c >> 10) as u32) * 255 / 31) as u8;
                    (r, g, b)
                };
                if r != 0 || g != 0 || b != 0 {
                    non_zero += 1;
                }
                pixels.push((r, g, b));
            }
        }
        println!("  non-zero pixels: {}/38400", non_zero);

        let bmp_path = format!("/tmp/test_{}.bmp", name);
        write_bmp(&bmp_path, &pixels, 240, 160).ok();
        let png_path = format!("/tmp/test_{}.png", name);
        std::process::Command::new("convert")
            .arg(&bmp_path)
            .arg(&png_path)
            .output()
            .ok();
        png_paths.push(png_path.clone());
    }

    println!("\n=== PNG files ===");
    for p in &png_paths {
        println!("{}", p);
    }
}
