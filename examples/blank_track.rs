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
        let map_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        let enabled = (dispcnt >> (8 + bg)) & 1;
        println!(
            "BG{}CNT={:04X} pri={} map={:#X} size={} h={:03X} v={:03X} en={}",
            bg, bgcnt, priority, map_base, size, hofs, vofs, enabled
        );
    }
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Same as full_capture
    for _ in 0..800u32 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("After START+A (frame 930):");
    print_bg_state(&gba);
    save_screenshot(&fb, "/tmp/fc3_after_start.png");

    // Press A
    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);

    // Track every frame until DISPCNT changes back from forced blank
    let mut last_dispcnt = 0x1F40u16;
    for i in 0..2000u32 {
        gba.run_frame_parallel(&mut fb);
        let dispcnt = gba.ppu().get_dispcnt();

        if dispcnt != last_dispcnt {
            println!(
                "\nFrame {} after A: DISPCNT {:04X} -> {:04X}",
                i, last_dispcnt, dispcnt
            );
            print_bg_state(&gba);

            // Save screenshot at state change
            if dispcnt != 0x0080 {
                save_screenshot(&fb, &format!("/tmp/fc3_frame_{}.png", i));

                // Dump BG map data
                let ppu = gba.ppu();
                let vram = ppu.vram();
                for bg in 0..4 {
                    let bgcnt = ppu.get_bgcnt(bg);
                    let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
                    let enabled = (dispcnt >> (8 + bg)) & 1;
                    if enabled != 0 && bgcnt != 0 {
                        println!("BG{} map at {:#X}:", bg, map_base);
                        for j in 0..5 {
                            let addr = map_base + j * 2;
                            if addr + 1 < vram.len() {
                                let val = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
                                println!("  [{}]={:04X}", j, val);
                            }
                        }
                    }
                }
            }
            last_dispcnt = dispcnt;
        }

        // Stop after we see a non-forced-blank screen
        if last_dispcnt != 0x0080 && last_dispcnt != 0x1F40 && i > 50 {
            break;
        }
    }
}
