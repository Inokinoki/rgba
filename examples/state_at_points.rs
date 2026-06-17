use rgba::Gba;
use std::fs;
use std::process::Command;

fn save_png(fb: &[u32], path: &str) {
    let ppm = path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let p = fb[y * 240 + x];
            bytes.push(((p >> 16) & 0xFF) as u8);
            bytes.push(((p >> 8) & 0xFF) as u8);
            bytes.push((p & 0xFF) as u8);
        }
    }
    fs::write(&ppm, &bytes).unwrap();
    Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm, path
            ),
        ])
        .output()
        .unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 500
    for _ in 0..500 {
        gba.run_frame_parallel(&mut fb);
    }

    {
        let ppu = gba.ppu();
        let dispcnt = ppu.get_dispcnt();
        println!("Frame 500: DISPCNT={:04X}", dispcnt);
    }
    save_png(&fb, "/tmp/bios_f500.png");

    let colors: std::collections::HashSet<u32> = fb.iter().copied().collect();
    println!("Frame 500: {} unique colors", colors.len());

    let mut black_count = 0;
    for &p in fb.iter() {
        if p == 0 {
            black_count += 1;
        }
    }
    println!(
        "Frame 500: {} black pixels out of {}",
        black_count,
        fb.len()
    );

    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    {
        let ppu = gba.ppu();
        println!(
            "\nAfter START (~frame 710): DISPCNT={:04X}",
            ppu.get_dispcnt()
        );
    }
    save_png(&fb, "/tmp/bios_after_start.png");

    let colors: std::collections::HashSet<u32> = fb.iter().copied().collect();
    println!("After START: {} unique colors", colors.len());

    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    {
        let ppu = gba.ppu();
        let dispcnt = ppu.get_dispcnt();
        println!("\nAfter START+A (dialogue): DISPCNT={:04X}", dispcnt);
        println!(
            "  BG0CNT={:04X} BG1CNT={:04X} BG2CNT={:04X} BG3CNT={:04X}",
            ppu.get_bgcnt(0),
            ppu.get_bgcnt(1),
            ppu.get_bgcnt(2),
            ppu.get_bgcnt(3)
        );
    }
    save_png(&fb, "/tmp/bios_dialogue.png");

    let colors: std::collections::HashSet<u32> = fb.iter().copied().collect();
    println!("After START+A: {} unique colors", colors.len());
}
