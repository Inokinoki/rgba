use rgba::{Gba, KeyState};

fn main() {
    let mut fb = vec![0u32; 240 * 160];

    // Test different key combos: what prevents auto-progression?
    let tests: Vec<(&str, Vec<KeyState>)> = vec![
        ("START only", vec![KeyState::START]),
        ("A only", vec![KeyState::A]),
        ("B only", vec![KeyState::B]),
        ("A+B", vec![KeyState::A, KeyState::B]),
        ("A+START", vec![KeyState::A, KeyState::START]),
        (
            "A+SELECT+START",
            vec![KeyState::A, KeyState::SELECT, KeyState::START],
        ),
    ];

    for (name, keys) in tests {
        let mut gba = Gba::new();
        gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
        gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
            .unwrap();

        for k in &keys {
            gba.input_mut().press_key(*k);
        }

        let mut auto_frame = 0u32;
        for f in 0..800 {
            let s = gba.mem.read_word(0x02000074);
            if s != 0 && s != 1 {
                auto_frame = f;
                break;
            }
            gba.run_frame_parallel(&mut fb);
        }
        println!(
            "{}: auto-progress at frame {}",
            name,
            if auto_frame > 0 {
                auto_frame.to_string()
            } else {
                "none (>800)".to_string()
            }
        );
    }

    // Test: press START only from frame 200 (after title screen visible)
    println!("\n=== START from frame 200 ===");
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().press_key(KeyState::START);
    for f in 0..600 {
        let s = gba.mem.read_word(0x02000074);
        if s != 0 && s != 1 {
            println!("  auto-progress at frame {}", 200 + f);
            break;
        }
        gba.run_frame_parallel(&mut fb);
    }
}
