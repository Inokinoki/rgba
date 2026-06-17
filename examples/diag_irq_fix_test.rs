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
    let vram_nonzero: usize = gba.mem().vram()[0..0xC000]
        .iter()
        .filter(|&&b| b != 0)
        .count();
    eprintln!(
        "{}: DC=0x{:04X} IE=0x{:04X} IF=0x{:04X} colors={} sprites={} vram={:.1}% PC=0x{:08X}",
        path,
        dc,
        ie,
        if_,
        unique_colors.len(),
        sprite_pixels,
        (vram_nonzero as f64 / 49152.0) * 100.0,
        gba.cpu_pc()
    );
}

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);

    let mut handler_addr: u32 = 0;
    let mut handler_seen = false;
    let mut vblank_count = 0u32;
    let mut prev_if_vblank = false;

    for i in 0..=2000u32 {
        let if_before = gba.mem().interrupt.if_raw.bits();
        let had_vblank_before = (if_before & 0x01) != 0;

        gba.run_frame();

        let if_after = gba.mem().interrupt.if_raw.bits();
        let has_vblank_after = (if_after & 0x01) != 0;

        if had_vblank_before && !has_vblank_after {
            vblank_count += 1;
        }

        if i == 5 {
            let iwram = &gba.mem().wram()[0..0x8000];
            let ptr =
                u32::from_le_bytes([iwram[0x7FFC], iwram[0x7FFD], iwram[0x7FFE], iwram[0x7FFF]]);
            eprintln!("Frame 5: IRQ handler pointer at 0x03007FFC = 0x{:08X}", ptr);
            handler_addr = ptr;
        }

        if !handler_seen && handler_addr != 0 {
            let pc = gba.cpu_pc();
            if pc == handler_addr || (pc & !1) == (handler_addr & !1) {
                eprintln!("Frame {}: IRQ handler executed! PC=0x{:08X}", i, pc);
                handler_seen = true;
            }
        }

        match i {
            3 | 5 | 10 | 20 | 50 | 100 | 200 | 300 | 500 | 700 | 1000 | 1500 | 2000 => {
                render_frame(&mut gba, &format!("/tmp/irq_fix_{:04}.bmp", i));
            }
            _ => {}
        }
    }

    eprintln!("\nTotal VBlank acknowledgements: {}", vblank_count);
    eprintln!("IRQ handler seen executing: {}", handler_seen);
}
