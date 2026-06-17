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

fn run_test(path: &str, name: &str) {
    let mut gba = Gba::new();
    if let Err(e) = gba.load_rom_path_patched(path) {
        println!("SKIP: {} ({})", name, e);
        return;
    }

    // Use run_frame_parallel which uses run_scanline (more accurate PPU)
    let mut fb = [0u32; 240 * 160];
    for _ in 0..600 {
        gba.run_frame_parallel(&mut fb);
    }

    let mut green = 0u32;
    let mut red = 0u32;
    let mut white = 0u32;
    let mut black = 0u32;
    let mut unique = std::collections::HashSet::new();

    for &pixel in &fb {
        let r = ((pixel >> 16) & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = (pixel & 0xFF) as u8;
        unique.insert(pixel);
        let r5 = r as u32 * 31 / 255;
        let g5 = g as u32 * 31 / 255;
        let b5 = b as u32 * 31 / 255;
        if g5 > 20 && r5 < 5 && b5 < 5 {
            green += 1;
        }
        if r5 > 20 && g5 < 5 && b5 < 5 {
            red += 1;
        }
        if r > 200 && g > 200 && b > 200 {
            white += 1;
        }
        if r < 30 && g < 30 && b < 30 {
            black += 1;
        }
    }

    // Save screenshot for debugging
    let pixels: Vec<(u8, u8, u8)> = fb
        .iter()
        .map(|&p| ((p >> 16) as u8, ((p >> 8) as u8), (p as u8)))
        .collect();
    write_bmp(&format!("/tmp/test_{}.bmp", name), &pixels, 240, 160).ok();

    let label = if green > 100 {
        "PASS"
    } else if red > 100 {
        "FAIL"
    } else {
        "UNKNOWN"
    };
    println!(
        "{}: {} (green={} red={} white={} black={} unique={} pc={:#010X})",
        label,
        name,
        green,
        red,
        white,
        black,
        unique.len(),
        gba.cpu().get_instruction_pc()
    );
}

fn main() {
    let tests = [
        ("/home/ubuntu/Builds/gba-tests/arm/arm.gba", "arm"),
        ("/home/ubuntu/Builds/gba-tests/thumb/thumb.gba", "thumb"),
        ("/home/ubuntu/Builds/gba-tests/unsafe/unsafe.gba", "unsafe"),
        ("/home/ubuntu/Builds/gba-tests/bios/bios.gba", "bios"),
        ("/home/ubuntu/Builds/gba-tests/memory/memory.gba", "memory"),
        ("/home/ubuntu/Builds/gba-tests/ppu/hello.gba", "hello"),
        ("/home/ubuntu/Builds/gba-tests/ppu/shades.gba", "shades"),
        ("/home/ubuntu/Builds/gba-tests/ppu/stripes.gba", "stripes"),
        ("/home/ubuntu/Builds/gba-tests/save/none.gba", "save-none"),
        ("/home/ubuntu/Builds/gba-tests/save/sram.gba", "save-sram"),
        (
            "/home/ubuntu/Builds/gba-tests/save/flash64.gba",
            "save-flash64",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/save/flash128.gba",
            "save-flash128",
        ),
        ("/home/ubuntu/Builds/gba-tests/nes/nes.gba", "nes"),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut unknown = 0;
    for (path, name) in &tests {
        run_test(path, name);
        // Count after printing
    }
}
