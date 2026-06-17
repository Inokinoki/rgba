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

fn count_tiles(vram: &[u8]) -> usize {
    let mut count = 0;
    for t in 0..512u16 {
        let start = t as usize * 32;
        for b in 0..32 {
            if vram[start + b] != 0 {
                count += 1;
                break;
            }
        }
    }
    count
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

    // Boot
    for _ in 0..3000 {
        gba.run_frame_parallel(&mut fb);
    }

    // START → menu
    press(&mut gba, rgba::KeyState::START, 300, &mut fb);
    // A → new game (assume first option)
    press(&mut gba, rgba::KeyState::A, 300, &mut fb);

    // Skip all name entry screens with RIGHT + A
    for screen in 0..4 {
        for _ in 0..40 {
            press(&mut gba, rgba::KeyState::RIGHT, 15, &mut fb);
        }
        press(&mut gba, rgba::KeyState::A, 200, &mut fb);
    }

    // Spam A through intro dialogue
    for _ in 0..100 {
        press(&mut gba, rgba::KeyState::A, 30, &mut fb);
    }

    // Check state
    gba.sync_ppu_full();
    let tiles = count_tiles(gba.ppu().vram());
    let dc = gba.ppu().get_dispcnt();
    println!("After intro: tiles={} DISPCNT={:#06X}", tiles, dc);
    let nw = render(&mut gba, "/tmp/farm2_00.bmp");
    println!("Screenshot: non_white={}", nw);

    // Now walk around the farm with D-pad to trigger tile loading
    let directions = [
        rgba::KeyState::DOWN,
        rgba::KeyState::RIGHT,
        rgba::KeyState::DOWN,
        rgba::KeyState::LEFT,
        rgba::KeyState::UP,
        rgba::KeyState::RIGHT,
        rgba::KeyState::DOWN,
    ];

    for walk in 0..14 {
        let dir = directions[walk % directions.len()];
        press(&mut gba, dir, 120, &mut fb);

        if walk % 2 == 1 {
            gba.sync_ppu_full();
            let tiles = count_tiles(gba.ppu().vram());
            let dc = gba.ppu().get_dispcnt();
            println!("Walk {}: tiles={} DISPCNT={:#06X}", walk + 1, tiles, dc);
        }
    }

    let nw = render(&mut gba, "/tmp/farm2_01.bmp");
    println!("After walking: non_white={}", nw);

    // Wait 5000 frames
    for _ in 0..5000 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();
    let tiles = count_tiles(gba.ppu().vram());
    println!("After wait: tiles={}", tiles);
    let nw = render(&mut gba, "/tmp/farm2_02.bmp");
    println!("Final: non_white={}", nw);

    // Convert
    for i in 0..=2 {
        let bmp = format!("/tmp/farm2_{:02}.bmp", i);
        let png = format!("/tmp/farm2_{:02}.png", i);
        std::process::Command::new("convert")
            .arg(&bmp)
            .arg(&png)
            .output()
            .ok();
    }
}
