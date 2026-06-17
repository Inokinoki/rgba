use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().swi_log.clear();

    for _ in 0..400 {
        gba.run_frame_parallel(&mut fb);
    }

    let swi_log = &gba.mem().swi_log;
    println!("SWI calls in 400 frames: {}", swi_log.len());

    let mut counts = std::collections::BTreeMap::new();
    for &swi in swi_log {
        *counts.entry(swi).or_insert(0) += 1;
    }
    for (swi, count) in &counts {
        println!("  SWI {:#04X}: {} calls", swi, count);
    }

    if !counts.contains_key(&0x10) && !counts.contains_key(&0x11) {
        println!("\n*** No LZ77 decompression (SWI 0x10/0x11) detected! ***");
        println!("This is likely why background tiles are not loaded.");
    }

    if let Some(&cnt) = counts.get(&0x0B) {
        println!("\nCpuSet (SWI 0x0B) called {} times", cnt);
    }
}
