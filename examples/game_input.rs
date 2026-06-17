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

fn render(gba: &mut Gba, path: &str) -> u32 {
    gba.sync_ppu_full();
    gba.sync_ppu();
    let mut pixels = Vec::new();
    let mut non_white = 0u32;
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = gba.get_pixel_tile_mode(x, y);
            let r = ((c & 0x1F) as u32 * 255 / 31) as u8;
            let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) as u8;
            let b = ((c >> 10) as u32 * 255 / 31) as u8;
            pixels.push((r, g, b));
            if c != 0x7FFF && c != 0 {
                non_white += 1;
            }
        }
    }
    write_bmp(path, &pixels, 240, 160).ok();
    non_white
}

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let mut gba = Gba::new();
    gba.load_rom_path(rom_path).unwrap();

    // Phase 1: Let game boot and settle (500 frames)
    for _ in 0..500 {
        gba.run_frame();
    }
    let nw = render(&mut gba, "/tmp/hm_phase1.bmp");
    println!(
        "Phase 1 (boot): non_white={} PC={:#010X}",
        nw,
        gba.cpu().get_instruction_pc()
    );

    // Phase 2: Wait longer, check if title screen appears (5000 frames)
    for _ in 0..4500 {
        gba.run_frame();
    }
    let nw = render(&mut gba, "/tmp/hm_phase2.bmp");
    println!(
        "Phase 2 (wait): non_white={} PC={:#010X}",
        nw,
        gba.cpu().get_instruction_pc()
    );

    // Phase 3: Press START to get past title
    gba.input.press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame();
    }
    gba.input.release_key(rgba::KeyState::START);
    for _ in 0..290 {
        gba.run_frame();
    }
    let nw = render(&mut gba, "/tmp/hm_phase3.bmp");
    println!(
        "Phase 3 (START): non_white={} PC={:#010X}",
        nw,
        gba.cpu().get_instruction_pc()
    );

    // Phase 4: Press A
    gba.input.press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame();
    }
    gba.input.release_key(rgba::KeyState::A);
    for _ in 0..990 {
        gba.run_frame();
    }
    let nw = render(&mut gba, "/tmp/hm_phase4.bmp");
    println!(
        "Phase 4 (A): non_white={} PC={:#010X}",
        nw,
        gba.cpu().get_instruction_pc()
    );

    // Phase 5: More A presses to get through dialogs
    for _ in 0..3 {
        gba.input.press_key(rgba::KeyState::A);
        for _ in 0..10 {
            gba.run_frame();
        }
        gba.input.release_key(rgba::KeyState::A);
        for _ in 0..290 {
            gba.run_frame();
        }
    }
    let nw = render(&mut gba, "/tmp/hm_phase5.bmp");
    println!(
        "Phase 5 (A x3): non_white={} PC={:#010X}",
        nw,
        gba.cpu().get_instruction_pc()
    );

    // Phase 6: Wait a long time
    for _ in 0..5000 {
        gba.run_frame();
    }
    let nw = render(&mut gba, "/tmp/hm_phase6.bmp");
    println!(
        "Phase 6 (wait): non_white={} PC={:#010X}",
        nw,
        gba.cpu().get_instruction_pc()
    );

    // Convert to PNG
    for phase in 1..=6 {
        let bmp = format!("/tmp/hm_phase{}.bmp", phase);
        let png = format!("/tmp/hm_phase{}.png", phase);
        std::process::Command::new("convert")
            .arg(&bmp)
            .arg(&png)
            .output()
            .ok();
    }
}
