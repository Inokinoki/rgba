use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..600 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check state over many frames
    let mut states: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for f in 0..400 {
        gba.run_frame_parallel(&mut fb);
        let s = gba.mem.read_word(0x02000074);
        *states.entry(s).or_insert(0) += 1;
        if f % 50 == 0 {
            println!("Frame {}: state={:08X}", 600 + f, s);
        }
        // Check if state ever returns to 1
        if s == 1 && f > 0 {
            println!("  -> Returned to title screen at frame {}", 600 + f);
        }
    }
    println!("\nState distribution:");
    for (s, count) in &states {
        println!("  {:08X}: {} frames", s, count);
    }
}
