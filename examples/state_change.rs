use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..560 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("=== Looking for state change around frame 560-580 ===");
    let mut prev: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for addr in (0x02000000..=0x02000100).step_by(4) {
        prev.insert(addr, gba.mem.read_word(addr));
    }

    for f in 0..30 {
        gba.run_frame_parallel(&mut fb);
        let mut changes = Vec::new();
        for addr in (0x02000000..=0x02000100).step_by(4) {
            let v = gba.mem.read_word(addr);
            let p = prev.get(&addr).copied().unwrap_or(0);
            if v != p {
                changes.push((addr, p, v));
                prev.insert(addr, v);
            }
        }
        if !changes.is_empty() {
            println!("Frame {}:", 561 + f);
            for (addr, old, new) in &changes {
                println!("  0x{:08X}: {:08X} -> {:08X}", addr, old, new);
            }
        }
    }
}
