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
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/save/flash128.gba")
        .unwrap();

    for _ in 0..500 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let vram = gba.ppu().vram();
    let dc = gba.ppu().get_dispcnt();
    println!("DISPCNT: {:#06X} mode={}", dc, dc & 0x7);

    // Read VRAM text from Mode 4 bitmap
    // The test ROMs use Mode 4 (0x0404) which is 8-bit paletted bitmap
    // Text is rendered as pixels in the framebuffer

    // Try reading screen as Mode 4
    let mut pixels = Vec::new();
    let pal = gba.mem().palette();
    for y in 0..160u16 {
        for x in 0..240u16 {
            let offset = (y as usize * 240 + x as usize);
            let color_idx = vram[offset] as usize;
            let color = u16::from_le_bytes([pal[color_idx * 2], pal[color_idx * 2 + 1]]);
            let r = ((color & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) as u8;
            let b = ((color >> 10) as u32 * 255 / 31) as u8;
            pixels.push((r, g, b));
        }
    }
    write_bmp("/tmp/flash128_debug.bmp", &pixels, 240, 160).ok();
    std::process::Command::new("convert")
        .arg("/tmp/flash128_debug.bmp")
        .arg("/tmp/flash128_debug.png")
        .output()
        .ok();

    // Check flash state by reading directly from SRAM
    println!("\nSRAM reads:");
    for addr in [
        0x0E000000u32,
        0x0E005555,
        0x0E002AAA,
        0x0E000100,
        0x0E001000,
    ] {
        let val = gba.mem_mut().read_byte(addr);
        println!("  {:#010X}: {:#04X}", addr, val);
    }

    println!("\nR12={:#010X}", gba.cpu().get_reg(12));
    println!("PC={:#010X}", gba.cpu().get_instruction_pc());
}
