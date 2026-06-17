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

fn press_release(gba: &mut Gba, key: rgba::KeyState, frames: usize) {
    gba.input.press_key(key);
    for _ in 0..frames {
        gba.run_frame();
    }
    gba.input.release_key(key);
    for _ in 0..frames {
        gba.run_frame();
    }
}

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    // Boot
    for _ in 0..1000 {
        gba.run_frame();
    }
    render(&mut gba, "/tmp/hm2_boot.bmp");
    println!("Boot: PC={:#010X}", gba.cpu().get_instruction_pc());

    // Press START (title screen -> name input?)
    press_release(&mut gba, rgba::KeyState::START, 10);
    for _ in 0..500 {
        gba.run_frame();
    }
    render(&mut gba, "/tmp/hm2_start.bmp");
    println!("After START: PC={:#010X}", gba.cpu().get_instruction_pc());

    // Press A many times to get through dialogs/name input
    for i in 0..20 {
        press_release(&mut gba, rgba::KeyState::A, 10);
        for _ in 0..100 {
            gba.run_frame();
        }
    }
    render(&mut gba, "/tmp/hm2_a20.bmp");
    println!("After 20x A: PC={:#010X}", gba.cpu().get_instruction_pc());

    // Press START again to try to skip/pause
    press_release(&mut gba, rgba::KeyState::START, 10);
    for _ in 0..500 {
        gba.run_frame();
    }
    render(&mut gba, "/tmp/hm2_start2.bmp");
    println!("After START2: PC={:#010X}", gba.cpu().get_instruction_pc());

    // More A presses
    for i in 0..20 {
        press_release(&mut gba, rgba::KeyState::A, 10);
        for _ in 0..100 {
            gba.run_frame();
        }
    }
    render(&mut gba, "/tmp/hm2_a40.bmp");
    println!("After 40x A: PC={:#010X}", gba.cpu().get_instruction_pc());

    // Wait a long time
    for _ in 0..3000 {
        gba.run_frame();
    }
    render(&mut gba, "/tmp/hm2_wait.bmp");
    println!("After wait: PC={:#010X}", gba.cpu().get_instruction_pc());

    for phase in ["boot", "start", "a20", "start2", "a40", "wait"] {
        let bmp = format!("/tmp/hm2_{}.bmp", phase);
        let png = format!("/tmp/hm2_{}.png", phase);
        std::process::Command::new("convert")
            .arg(&bmp)
            .arg(&png)
            .output()
            .ok();
    }
}
