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

fn press(gba: &mut Gba, key: rgba::KeyState, frames: u32, fb: &mut [u32]) {
    gba.input_mut().press_key(key);
    for _ in 0..10 {
        gba.run_frame_parallel(fb);
    }
    gba.input_mut().release_key(key);
    for _ in 0..(frames.saturating_sub(10)) {
        gba.run_frame_parallel(fb);
    }
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Boot
    for _ in 0..3000 {
        gba.run_frame_parallel(&mut fb);
    }
    let nw = render(&mut gba, "/tmp/nav_00_title.bmp");
    println!("Title: non_white={}", nw);

    // Press START
    press(&mut gba, rgba::KeyState::START, 300, &mut fb);
    let nw = render(&mut gba, "/tmp/nav_01_menu.bmp");
    println!("Menu: non_white={}", nw);

    // Spam A to get through name entry and menus
    for i in 0..50 {
        press(&mut gba, rgba::KeyState::A, 120, &mut fb);
        if i % 10 == 9 {
            let path = format!("/tmp/nav_{:02}.bmp", i / 10 + 2);
            let nw = render(&mut gba, &path);
            println!("A press {}: non_white={}", i + 1, nw);
        }
    }

    // Wait longer for game to settle
    for _ in 0..1000 {
        gba.run_frame_parallel(&mut fb);
    }
    let nw = render(&mut gba, "/tmp/nav_07_settled.bmp");
    println!("Settled: non_white={}", nw);

    // Try more A presses to advance
    for i in 0..30 {
        press(&mut gba, rgba::KeyState::A, 120, &mut fb);
    }
    let nw = render(&mut gba, "/tmp/nav_08_more_a.bmp");
    println!("More A: non_white={}", nw);

    // Wait long
    for _ in 0..3000 {
        gba.run_frame_parallel(&mut fb);
    }
    let nw = render(&mut gba, "/tmp/nav_09_wait.bmp");
    println!("Wait: non_white={}", nw);

    // Convert all to PNG
    for f in std::fs::read_dir("/tmp").unwrap() {
        let f = f.unwrap();
        let name = f.file_name().to_string_lossy().to_string();
        if name.starts_with("nav_") && name.ends_with(".bmp") {
            let png = name.replace(".bmp", ".png");
            std::process::Command::new("convert")
                .arg(format!("/tmp/{}", name))
                .arg(format!("/tmp/{}", png))
                .output()
                .ok();
        }
    }
}
