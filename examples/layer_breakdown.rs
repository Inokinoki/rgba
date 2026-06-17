use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = gba.ppu();
    let vram = ppu.vram();
    let pal = gba.mem().palette();

    let dispcnt = ppu.get_dispcnt();
    println!("DISPCNT: {:04X}", dispcnt);
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        println!(
            "BG{}CNT: {:04X} (pri={}, char_base={:05X}, screen_base={:05X}, {}bpp, size={})",
            bg,
            bgcnt,
            bgcnt & 3,
            ((bgcnt >> 2) & 0x3) as usize * 0x4000,
            ((bgcnt >> 8) & 0x1F) as usize * 0x800,
            if (bgcnt & 0x80) != 0 { "8" } else { "4" },
            (bgcnt >> 14) & 3
        );
        println!(
            "BG{}HOFS: {}  BG{}VOFS: {}",
            bg,
            ppu.get_bg_hofs(bg),
            bg,
            ppu.get_bg_vofs(bg)
        );
    }

    // Count pixels per BG layer
    let mut bg_pixels = [0u32; 4];
    let mut backdrop_pixels = 0u32;
    let mut sprite_pixels = 0u32;

    for y in 0..160u16 {
        for x in 0..240u16 {
            let mut contributing_bg = 5u8;
            for bg in 0..4 {
                if dispcnt & (1 << (8 + bg)) == 0 {
                    continue;
                }
                if let Some(_color) = gba.get_bg_pixel(ppu, (dispcnt & 7) as u8, bg, x, y) {
                    if contributing_bg == 5 {
                        contributing_bg = bg as u8;
                    }
                }
            }
            // Check sprites
            let has_sprite = if let Some((_, _)) = gba.get_sprite_pixel(ppu, x, y) {
                true
            } else {
                false
            };

            if has_sprite {
                sprite_pixels += 1;
            } else if contributing_bg < 4 {
                bg_pixels[contributing_bg as usize] += 1;
            } else {
                backdrop_pixels += 1;
            }
        }
    }

    println!("\nPixel source breakdown:");
    for bg in 0..4 {
        println!(
            "  BG{}: {} pixels ({:.1}%)",
            bg,
            bg_pixels[bg],
            bg_pixels[bg] as f64 / 38400.0 * 100.0
        );
    }
    println!(
        "  Sprites: {} pixels ({:.1}%)",
        sprite_pixels,
        sprite_pixels as f64 / 38400.0 * 100.0
    );
    println!(
        "  Backdrop: {} pixels ({:.1}%)",
        backdrop_pixels,
        backdrop_pixels as f64 / 38400.0 * 100.0
    );

    // Save a version where backdrop is black for better visibility
    let mut fb2 = vec![0u32; 240 * 160];
    for y in 0..160usize {
        for x in 0..240usize {
            let pixel = fb[y * 240 + x];
            // Keep the pixel unless it's the green backdrop
            let r = (pixel >> 16) & 0xFF;
            let g = (pixel >> 8) & 0xFF;
            let b = pixel & 0xFF;
            if r == 0 && g == 255 && b == 0 {
                fb2[y * 240 + x] = 0x00202020; // Dark gray instead of green
            } else {
                fb2[y * 240 + x] = pixel;
            }
        }
    }

    // Save as PPM
    let mut out = Vec::new();
    out.extend_from_slice(b"P6\n240 160\n255\n");
    for y in 0..160usize {
        for x in 0..240usize {
            let p = fb2[y * 240 + x];
            out.extend_from_slice(&[((p >> 16) as u8), ((p >> 8) as u8), (p as u8)]);
        }
    }
    std::fs::write("/tmp/game_nobg.ppm", &out).unwrap();

    // Also save a version showing only BG3 (the top-priority layer)
    let mut fb3 = vec![0u32; 240 * 160];
    for y in 0..160u16 {
        for x in 0..240u16 {
            if let Some(color) = gba.get_bg_pixel(ppu, 0, 3, x, y) {
                let r = ((color & 0x1F) as u32 * 255 / 31) << 16;
                let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
                let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                fb3[y as usize * 240 + x as usize] = r | g | b;
            }
        }
    }
    let mut out3 = Vec::new();
    out3.extend_from_slice(b"P6\n240 160\n255\n");
    for y in 0..160usize {
        for x in 0..240usize {
            let p = fb3[y * 240 + x];
            out3.extend_from_slice(&[((p >> 16) as u8), ((p >> 8) as u8), (p as u8)]);
        }
    }
    std::fs::write("/tmp/game_bg3.ppm", &out3).unwrap();

    println!("\nSaved /tmp/game_nobg.ppm and /tmp/game_bg3.ppm");
}
