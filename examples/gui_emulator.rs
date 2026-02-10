//! RGBA GBA Emulator - GUI Application
//!
//! A graphical emulator interface using minifb.

#[cfg(feature = "gui")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::time::Duration;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let rom_path = if args.len() > 1 {
        Some(&args[1])
    } else {
        println!("RGBA GBA Emulator");
        println!("Usage: {} <rom_file>", args[0]);
        println!();
        println!("Starting without ROM - showing test pattern...");
        println!();
        None
    };

    // Create GBA emulator
    let mut gba = rgba::Gba::new();

    // Load ROM if provided
    if let Some(path) = rom_path {
        match gba.load_rom_path(path) {
            Ok(_) => println!("Loaded ROM: {}", path),
            Err(e) => {
                eprintln!("Failed to load ROM: {}", e);
                eprintln!("Starting without ROM...");
            }
        }
    }

    // Window dimensions - scale up by 3x
    let width = 240;
    let height = 160;
    let scale = 3;
    let window_width = width * scale;
    let window_height = height * scale;

    // Create window
    let title = format!("RGBA GBA Emulator - {}",
        rom_path.unwrap_or(&"No ROM".to_string()).clone()
    );
    let mut window = minifb::Window::new(
        &title,
        window_width,
        window_height,
        minifb::WindowOptions::default()
    ).unwrap_or_else(|e| {
        eprintln!("Failed to create window: {}", e);
        std::process::exit(1);
    });

    // Limit to 60 FPS
    window.limit_update_rate(Some(std::time::Duration::from_nanos(1_000_000_000 / 60)));

    // Frame buffer
    let mut buffer: Vec<u32> = vec![0; width * height];

    let mut is_running = true;
    let mut fps_counter = 0;
    let mut last_time = std::time::Instant::now();
    let mut fps = 0;

    // Main loop
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        // Update FPS counter
        fps_counter += 1;
        if last_time.elapsed() >= Duration::from_secs(1) {
            fps = fps_counter;
            fps_counter = 0;
            last_time = std::time::Instant::now();
            let title = format!("RGBA GBA Emulator - {} FPS - {}",
                fps,
                rom_path.unwrap_or(&"No ROM".to_string()).clone()
            );
            window.set_title(&title);
        }

        // Handle input
        use rgba::KeyState;

        // D-Pad
        gba.input_mut().release_key(KeyState::UP);
        gba.input_mut().release_key(KeyState::DOWN);
        gba.input_mut().release_key(KeyState::LEFT);
        gba.input_mut().release_key(KeyState::RIGHT);

        if window.is_key_down(minifb::Key::Up) {
            gba.input_mut().press_key(KeyState::UP);
        }
        if window.is_key_down(minifb::Key::Down) {
            gba.input_mut().press_key(KeyState::DOWN);
        }
        if window.is_key_down(minifb::Key::Left) {
            gba.input_mut().press_key(KeyState::LEFT);
        }
        if window.is_key_down(minifb::Key::Right) {
            gba.input_mut().press_key(KeyState::RIGHT);
        }

        // Buttons
        gba.input_mut().release_key(KeyState::A);
        gba.input_mut().release_key(KeyState::B);
        gba.input_mut().release_key(KeyState::START);
        gba.input_mut().release_key(KeyState::SELECT);
        gba.input_mut().release_key(KeyState::L);
        gba.input_mut().release_key(KeyState::R);

        if window.is_key_down(minifb::Key::Z) {
            gba.input_mut().press_key(KeyState::A);
        }
        if window.is_key_down(minifb::Key::X) {
            gba.input_mut().press_key(KeyState::B);
        }
        if window.is_key_down(minifb::Key::Enter) {
            gba.input_mut().press_key(KeyState::START);
        }
        if window.is_key_down(minifb::Key::RightShift) {
            gba.input_mut().press_key(KeyState::SELECT);
        }
        if window.is_key_down(minifb::Key::A) {
            gba.input_mut().press_key(KeyState::L);
        }
        if window.is_key_down(minifb::Key::S) {
            gba.input_mut().press_key(KeyState::R);
        }

        // Control keys
        if window.is_key_down(minifb::Key::P) {
            // Toggle pause - requires key release detection
            // For now, just continue
        }
        if window.is_key_down(minifb::Key::R) {
            gba.reset();
        }

        // Run emulation
        if is_running {
            // Run one frame (approximately 2800 steps for 60 FPS)
            for _ in 0..2800 {
                gba.step();
            }
        }

        // Render the screen
        let ppu = gba.ppu();
        let mode = ppu.get_display_mode();

        for y in 0..height {
            for x in 0..width {
                let color = match mode {
                    0 | 1 | 2 => {
                        // Tile/text modes
                        let c = gba.get_pixel_tile_mode(x as u16, y as u16);
                        let r = ((c >> 0) & 0x1F) as u8;
                        let g = ((c >> 5) & 0x1F) as u8;
                        let b = ((c >> 10) & 0x1F) as u8;
                        ((r as u32 * 255 / 31) << 16) |
                        ((g as u32 * 255 / 31) << 8) |
                        (b as u32 * 255 / 31)
                    }
                    3 => {
                        // Mode 3: 16-bit bitmap
                        let c = ppu.get_pixel_mode3(x as u16, y as u16);
                        let r = ((c >> 0) & 0x1F) as u8;
                        let g = ((c >> 5) & 0x1F) as u8;
                        let b = ((c >> 10) & 0x1F) as u8;
                        ((r as u32 * 255 / 31) << 16) |
                        ((g as u32 * 255 / 31) << 8) |
                        (b as u32 * 255 / 31)
                    }
                    4 => {
                        // Mode 4: 8-bit paletted
                        let idx = ppu.get_pixel_mode4(x as u16, y as u16) as u32;
                        // Get actual palette color
                        let c = gba.get_palette_color(0, idx as u16);
                        let r = ((c >> 0) & 0x1F) as u8;
                        let g = ((c >> 5) & 0x1F) as u8;
                        let b = ((c >> 10) & 0x1F) as u8;
                        ((r as u32 * 255 / 31) << 16) |
                        ((g as u32 * 255 / 31) << 8) |
                        (b as u32 * 255 / 31)
                    }
                    _ => {
                        // Test pattern for unsupported modes
                        let r = (x * 255 / width) as u32;
                        let g = (y * 255 / height) as u32;
                        let b = 128u32;
                        (r << 16) | (g << 8) | b
                    }
                };
                buffer[y * width + x] = color;
            }
        }

        // Update window
        window.update_with_buffer(&buffer, width, height).unwrap();
    }

    Ok(())
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("Error: GUI feature not enabled!");
    eprintln!("Please run with: cargo run --example gui_emulator --features gui -- <rom_file>");
    std::process::exit(1);
}
