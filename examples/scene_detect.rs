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
    println!("DISPCNT={:04X}", dispcnt);
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let priority = bgcnt & 3;
        let tile_base = ((bgcnt >> 2) & 3) * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        println!(
            "BG{}CNT={:04X} pri={} tile_base={:#X} map_base={:#X} size={} HOFS={:03X} VOFS={:03X}",
            bg, bgcnt, priority, tile_base, map_base, size, hofs, vofs
        );
    }
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    let mut frame = 0u32;
    let mut last_bg0cnt = 0u16;
    let mut screenshots_taken = 0u32;

    // Run until we detect scene transitions (BGCNT changes)
    for _ in 0..2000u32 {
        gba.run_frame_parallel(&mut fb);
        frame += 1;

        let bg0cnt = gba.ppu().get_bgcnt(0);
        if bg0cnt != last_bg0cnt && last_bg0cnt != 0 {
            println!("\n=== BG0CNT changed at frame {} ===", frame);
            println!(
                "  Old BG0CNT={:04X}, New BG0CNT={:04X}",
                last_bg0cnt, bg0cnt
            );
            print_bg_state(&gba);

            let vram = gba.ppu().vram();
            let map_base = ((bg0cnt >> 8) & 0x1F) as usize * 0x800;
            println!("BG0 map entries at {:#X}:", map_base);
            for i in 0..10 {
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

            if screenshots_taken < 20 {
                save_screenshot(&fb, &format!("/tmp/scene_{}.png", frame));
                println!("Saved scene_{}.png", frame);
                screenshots_taken += 1;
            }
        }
        last_bg0cnt = bg0cnt;

        // At frame 300, press START
        if frame == 300 {
            println!("\n=== Pressing START at frame 300 ===");
            gba.input_mut().press_key(rgba::KeyState::START);
        }
        if frame == 310 {
            gba.input_mut().release_key(rgba::KeyState::START);
            println!("Released START at frame 310");
        }

        // After START transition settles, press A
        if frame == 700 {
            println!("\n=== Pressing A at frame 700 ===");
            gba.input_mut().press_key(rgba::KeyState::A);
        }
        if frame == 710 {
            gba.input_mut().release_key(rgba::KeyState::A);
            println!("Released A at frame 710");
        }
    }

    println!(
        "\nTotal frames: {}, screenshots: {}",
        frame, screenshots_taken
    );
}
