use rgba::Gba;
use std::fs;

fn save_screenshot(fb: &[u32], png_path: &str) {
    let ppm_path = png_path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            bytes.push(((pixel >> 16) & 0xFF) as u8);
            bytes.push(((pixel >> 8) & 0xFF) as u8);
            bytes.push((pixel & 0xFF) as u8);
        }
    }
    fs::write(&ppm_path, &bytes).unwrap();
    std::process::Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm_path, png_path
            ),
        ])
        .output()
        .unwrap();
}

fn print_bg_state(gba: &Gba) {
    let ppu = gba.ppu();
    let dispcnt = ppu.get_dispcnt();
    println!(
        "DISPCNT={:04X} (mode={} BGs={} OBJ={})",
        dispcnt,
        dispcnt & 7,
        (dispcnt >> 8) & 0xF,
        (dispcnt >> 12) & 1
    );
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let priority = bgcnt & 3;
        let tile_base = ((bgcnt >> 2) & 3) * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        let enabled = (dispcnt >> (8 + bg)) & 1;
        println!(
            "BG{}CNT={:04X} pri={} tile={:#X} map={:#X} size={} h={:03X} v={:03X} en={}",
            bg, bgcnt, priority, tile_base, map_base, size, hofs, vofs, enabled
        );
    }
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Press START (same as full_capture)
    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120u32 {
        gba.run_frame_parallel(&mut fb);
    }

    save_screenshot(&fb, "/tmp/fc_after_title.png");
    println!("=== After title (frame 630) ===");
    print_bg_state(&gba);

    // Now press A
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);

    // Capture every frame for the next 130 frames
    for i in 0..130u32 {
        gba.run_frame_parallel(&mut fb);

        let ppu = gba.ppu();
        let dispcnt = ppu.get_dispcnt();
        let bg0cnt = ppu.get_bgcnt(0);

        // Only print on interesting frames
        if i < 5 || i % 10 == 0 || dispcnt != 0x1F40 {
            println!("\n--- A+release + {} frames ---", i);
            print_bg_state(&gba);

            // Check first few pixels
            let first_pixel = fb[120 * 240 + 120]; // center pixel
            println!("Center pixel: {:08X}", first_pixel);
        }

        if i == 10 || i == 20 || i == 50 || i == 120 {
            save_screenshot(&fb, &format!("/tmp/fc_dialogue_{:03}.png", i));
        }
    }

    // Let's also check: does the game actually switch modes after START+A?
    // Try the full_capture sequence
    println!("\n\n=== RESTART: full_capture sequence ===");
    let mut gba2 = Gba::new();
    gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb2 = vec![0u32; 240 * 160];

    let mut frame = 0u32;
    // Same as full_capture
    for t in [300, 500, 800].iter() {
        while frame < *t {
            gba2.run_frame_parallel(&mut fb2);
            frame += 1;
        }
    }

    // START
    gba2.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba2.run_frame_parallel(&mut fb2);
        frame += 1;
    }
    gba2.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120 {
        gba2.run_frame_parallel(&mut fb2);
        frame += 1;
    }

    println!("After title (frame {}):", frame);
    print_bg_state(&gba2);

    // Press A
    gba2.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba2.run_frame_parallel(&mut fb2);
        frame += 1;
    }
    gba2.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..120 {
        gba2.run_frame_parallel(&mut fb2);
        frame += 1;
    }

    println!("\nDialogue_0 (frame {}):", frame);
    print_bg_state(&gba2);
    save_screenshot(&fb2, "/tmp/fc2_dialogue0.png");

    let ppu = gba2.ppu();
    let vram = ppu.vram();

    // Check what BG1 is showing (since DISPCNT=1640 means BG1 and BG2 enabled)
    let bg1cnt = ppu.get_bgcnt(1);
    let map_base = ((bg1cnt >> 8) & 0x1F) as usize * 0x800;
    println!("\nBG1 map at {:#X}:", map_base);
    for i in 0..32 {
        let addr = map_base + i * 2;
        if addr + 1 < vram.len() {
            let val = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
            println!(
                "  entry[{}] = {:04X} (tile={} pal={})",
                i,
                val,
                val & 0x3FF,
                (val >> 12) & 0xF
            );
        }
    }

    // Check BG2 map
    let bg2cnt = ppu.get_bgcnt(2);
    let map_base2 = ((bg2cnt >> 8) & 0x1F) as usize * 0x800;
    println!("\nBG2 map at {:#X}:", map_base2);
    for i in 0..32 {
        let addr = map_base2 + i * 2;
        if addr + 1 < vram.len() {
            let val = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
            println!(
                "  entry[{}] = {:04X} (tile={} pal={})",
                i,
                val,
                val & 0x3FF,
                (val >> 12) & 0xF
            );
        }
    }

    // Check tile data at tile 418 (used in BG1 map entries)
    let tile_base = ((bg1cnt >> 2) & 3) as usize * 0x4000;
    println!("\nTile 418 data at tile_base={:#X}:", tile_base);
    let tile_addr = tile_base + 418 * 32;
    for row in 0..8 {
        let mut pixels = String::new();
        for col in 0..4 {
            let byte = vram[tile_addr + row * 4 + col];
            let lo = byte & 0xF;
            let hi = (byte >> 4) & 0xF;
            pixels.push_str(&format!("{:X}{:X}", lo, hi));
        }
        println!("  row{}: {}", row, pixels);
    }

    // Check palette 0
    println!("\nBG palette bank 0 (entries 0-15):");
    for i in 0..16u16 {
        let color = gba2.get_palette_color(0, i);
        if color != 0 {
            let r = color & 0x1F;
            let g = (color >> 5) & 0x1F;
            let b = (color >> 10) & 0x1F;
            println!("  pal[{}] = {:04X} (r={} g={} b={})", i, color, r, g, b);
        }
    }
}
