use rgba::Gba;
use std::io::Write;

fn rgb555_to_u8(color: u16) -> (u8, u8, u8) {
    let r = ((color & 0x1F) * 255 / 31) as u8;
    let g = (((color >> 5) & 0x1F) * 255 / 31) as u8;
    let b = (((color >> 10) & 0x1F) * 255 / 31) as u8;
    (r, g, b)
}

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

fn print_usage() {
    eprintln!("Usage: rgba <rom_path> [options]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --bios <path>       Load BIOS from file");
    eprintln!("  --frames <N>        Number of frames to run (default: 60)");
    eprintln!("  --output <path>     BMP output path (default: output.bmp)");
    eprintln!("  --save-type <type>  Save type: sram, flash64, flash128, eeprom512, eeprom8k");
    #[cfg(feature = "gui")]
    eprintln!("  --gui               Run with graphical window");
    eprintln!("  --help              Show this help");
}

fn parse_args(args: &[String]) -> Option<(String, Option<String>, u32, String, Option<String>)> {
    let mut rom_path = None;
    let mut bios_path = None;
    let mut frames = 60u32;
    let mut output = "output.bmp".to_string();
    let mut save_type = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--bios" => {
                i += 1;
                bios_path = args.get(i).cloned();
            }
            "--frames" => {
                i += 1;
                frames = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(60);
            }
            "--output" => {
                i += 1;
                output = args.get(i).cloned().unwrap_or_else(|| "output.bmp".to_string());
            }
            "--save-type" => {
                i += 1;
                save_type = args.get(i).cloned();
            }
            #[cfg(feature = "gui")]
            "--gui" => {}
            s if !s.starts_with('-') => {
                rom_path = Some(s.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    rom_path.map(|r| (r, bios_path, frames, output, save_type))
}

fn run_headless(args: &[String]) {
    let (rom_path, bios_path, frames, output, save_type) = match parse_args(args) {
        Some(v) => v,
        None => {
            print_usage();
            std::process::exit(1);
        }
    };

    let rom_data = std::fs::read(&rom_path).unwrap_or_else(|e| {
        eprintln!("Error reading ROM '{}': {}", rom_path, e);
        std::process::exit(1);
    });
    println!("Loaded ROM: {} bytes ({})", rom_data.len(), rom_path);

    let mut gba = Gba::new();

    if let Some(ref bios) = bios_path {
        if let Err(e) = gba.load_bios_path(bios) {
            eprintln!("Warning: Failed to load BIOS '{}': {}", bios, e);
        } else {
            println!("Loaded BIOS: {}", bios);
        }
    }

    if let Some(ref st) = save_type {
        apply_save_type(&mut gba, st);
    }

    gba.load_rom(rom_data);

    println!("Running {} frames...", frames);
    for i in 0..frames {
        gba.run_frame();
        if i > 0 && i % 60 == 0 {
            println!("  Frame {} (PC=0x{:08X})", i, gba.cpu_pc());
        }
    }

    // Render final frame to BMP
    gba.sync_ppu_full();
    let width = 240u32;
    let height = 160u32;
    let mut pixels = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            let color = gba.get_pixel_tile_mode(x as u16, y as u16);
            pixels.push(rgb555_to_u8(color));
        }
    }

    match write_bmp(&output, &pixels, width, height) {
        Ok(()) => println!("Saved {}x{} frame to {}", width, height, output),
        Err(e) => eprintln!("Error writing BMP: {}", e),
    }

    println!("Final PC: 0x{:08X}", gba.cpu_pc());
}

fn apply_save_type(gba: &mut Gba, st: &str) {
    use rgba::SaveType;
    let save = match st {
        "sram" => SaveType::Sram,
        "flash64" => SaveType::Flash64K,
        "flash128" => SaveType::Flash128K,
        "eeprom512" => SaveType::Eeprom512B,
        "eeprom8k" => SaveType::Eeprom8K,
        _ => {
            eprintln!(
                "Unknown save type '{}'. Use: sram, flash64, flash128, eeprom512, eeprom8k",
                st
            );
            std::process::exit(1);
        }
    };
    gba.set_save_type(save);
    println!("Save type: {:?}", save);
}

#[cfg(feature = "gui")]
fn run_gui(args: &[String]) {
    use minifb::{Key, Scale, Window, WindowOptions};
    use rgba::KeyState;

    let (rom_path, bios_path, _frames, _output, save_type) = match parse_args(args) {
        Some(v) => v,
        None => {
            print_usage();
            std::process::exit(1);
        }
    };

    let rom_data = std::fs::read(&rom_path).unwrap_or_else(|e| {
        eprintln!("Error reading ROM '{}': {}", rom_path, e);
        std::process::exit(1);
    });
    println!("Loaded ROM: {} bytes ({})", rom_data.len(), rom_path);

    let mut gba = Gba::new();

    if let Some(ref bios) = bios_path {
        if let Err(e) = gba.load_bios_path(bios) {
            eprintln!("Warning: Failed to load BIOS '{}': {}", bios, e);
        }
    }

    if let Some(ref st) = save_type {
        apply_save_type(&mut gba, st);
    }

    gba.load_rom(rom_data);

    let width = 240;
    let height = 160;
    let scale = 3;
    let mut window = Window::new(
        "RGBA - GBA Emulator",
        width * scale as usize,
        height * scale as usize,
        WindowOptions {
            scale: Scale::X3,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        eprintln!("Failed to create window: {}", e);
        std::process::exit(1);
    });

    let mut buffer = vec![0u32; (width * height * scale * scale) as usize];

    let mut frame_count = 0u64;
    let mut fps_timer = std::time::Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle input
        let key_map = [
            (Key::Up, KeyState::UP),
            (Key::Down, KeyState::DOWN),
            (Key::Left, KeyState::LEFT),
            (Key::Right, KeyState::RIGHT),
            (Key::Z, KeyState::A),
            (Key::X, KeyState::B),
            (Key::Enter, KeyState::START),
            (Key::RightShift, KeyState::SELECT),
            (Key::A, KeyState::L),
            (Key::S, KeyState::R),
        ];

        for (kb_key, gba_key) in &key_map {
            if window.is_key_down(*kb_key) {
                gba.input_mut().press_key(*gba_key);
            } else {
                gba.input_mut().release_key(*gba_key);
            }
        }

        // Reset
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            gba.reset();
        }

        // Run one frame
        gba.run_frame();

        // Render
        gba.sync_ppu_full();
        let mut idx = 0;
        for y in 0..height as usize {
            for x in 0..width as usize {
                let color = gba.get_pixel_tile_mode(x as u16, y as u16);
                let r = ((color & 0x1F) * 255 / 31) as u32;
                let g = (((color >> 5) & 0x1F) * 255 / 31) as u32;
                let b = (((color >> 10) & 0x1F) * 255 / 31) as u32;
                let rgb = (r << 16) | (g << 8) | b;

                // Scale up
                let sy = y * scale as usize;
                let sx = x * scale as usize;
                let screen_w = width as usize * scale as usize;
                for dy in 0..scale as usize {
                    for dx in 0..scale as usize {
                        buffer[(sy + dy) * screen_w + sx + dx] = rgb;
                    }
                }
                idx += 1;
            }
        }

        window.update_with_buffer(&buffer, width as usize * scale as usize, height as usize * scale as usize).unwrap();

        // FPS counter
        frame_count += 1;
        if fps_timer.elapsed() >= std::time::Duration::from_secs(1) {
            let fps = frame_count as f64 / fps_timer.elapsed().as_secs_f64();
            window.set_title(&format!("RGBA - GBA Emulator ({:.1} FPS)", fps));
            frame_count = 0;
            fps_timer = std::time::Instant::now();
        }

        // Frame rate limiting (~60 FPS)
        std::thread::sleep(std::time::Duration::from_micros(16000));
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check for --gui flag
    #[cfg(feature = "gui")]
    {
        if args.iter().any(|a| a == "--gui") {
            run_gui(&args);
            return;
        }
    }

    // Default: headless mode
    run_headless(&args);
}
