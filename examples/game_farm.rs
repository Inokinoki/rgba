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
    for _ in 0..frames.saturating_sub(10) {
        gba.run_frame_parallel(fb);
    }
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Boot to title
    for _ in 0..3000 {
        gba.run_frame_parallel(&mut fb);
    }
    render(&mut gba, "/tmp/farm_00.bmp");

    // Press START to get to menu
    press(&mut gba, rgba::KeyState::START, 300, &mut fb);
    render(&mut gba, "/tmp/farm_01.bmp");

    // Press A to select "New Game" (assuming first option)
    press(&mut gba, rgba::KeyState::A, 300, &mut fb);
    render(&mut gba, "/tmp/farm_02.bmp");

    // Name entry screen - navigate to end and confirm
    // In HM, the name entry has a grid. Navigate right many times to reach "End/OK"
    for _ in 0..30 {
        press(&mut gba, rgba::KeyState::RIGHT, 30, &mut fb);
    }
    // Press A to select "OK" or confirm
    press(&mut gba, rgba::KeyState::A, 200, &mut fb);
    render(&mut gba, "/tmp/farm_03.bmp");

    // Farm name entry - same approach
    for _ in 0..30 {
        press(&mut gba, rgba::KeyState::RIGHT, 30, &mut fb);
    }
    press(&mut gba, rgba::KeyState::A, 200, &mut fb);
    render(&mut gba, "/tmp/farm_04.bmp");

    // Birthday selection - might need UP/DOWN
    for _ in 0..5 {
        press(&mut gba, rgba::KeyState::RIGHT, 30, &mut fb);
    }
    press(&mut gba, rgba::KeyState::A, 200, &mut fb);
    render(&mut gba, "/tmp/farm_05.bmp");

    // Dog name entry - same approach
    for _ in 0..30 {
        press(&mut gba, rgba::KeyState::RIGHT, 30, &mut fb);
    }
    press(&mut gba, rgba::KeyState::A, 200, &mut fb);
    render(&mut gba, "/tmp/farm_06.bmp");

    // Intro dialogue - spam A to advance
    for i in 0..60 {
        press(&mut gba, rgba::KeyState::A, 60, &mut fb);
    }
    let nw = render(&mut gba, "/tmp/farm_07.bmp");
    println!("After dialogue spam: non_white={}", nw);

    // Try START to skip any remaining menus
    press(&mut gba, rgba::KeyState::START, 300, &mut fb);
    render(&mut gba, "/tmp/farm_08.bmp");

    // More A presses
    for i in 0..30 {
        press(&mut gba, rgba::KeyState::A, 60, &mut fb);
    }
    let nw = render(&mut gba, "/tmp/farm_09.bmp");
    println!("Final: non_white={}", nw);

    // Wait and check
    for _ in 0..3000 {
        gba.run_frame_parallel(&mut fb);
    }
    let nw = render(&mut gba, "/tmp/farm_10.bmp");
    println!("After wait: non_white={}", nw);

    gba.sync_ppu_full();
    let dc = gba.ppu().get_dispcnt();
    println!("Final DISPCNT: {:#06X}", dc);

    // Convert
    for i in 0..=10 {
        let bmp = format!("/tmp/farm_{:02}.bmp", i);
        let png = format!("/tmp/farm_{:02}.png", i);
        std::process::Command::new("convert")
            .arg(&bmp)
            .arg(&png)
            .output()
            .ok();
    }
}
