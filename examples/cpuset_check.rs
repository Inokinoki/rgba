use rgba::Gba;
use rgba::KeyState;
fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];
    gba.mem_mut().cpu_set_log_enabled = true;
    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); }
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); }
    }
    let log = &gba.mem().cpu_set_log;
    println!("Total CpuSet calls: {}", log.len());
    let mut targets: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    for &(src, dst, cnt) in log {
        let region = if dst >= 0x06000000 && dst < 0x06018000 { "VRAM" }
            else if dst >= 0x05000000 && dst < 0x05000400 { "Palette" }
            else if dst >= 0x07000000 && dst < 0x07000400 { "OAM" }
            else if dst >= 0x02000000 && dst < 0x03000000 { "EWRAM" }
            else if dst >= 0x03000000 && dst < 0x04000000 { "IWRAM" }
            else if dst >= 0x08000000 { "ROM" }
            else { "Other" };
        *targets.entry(region.to_string()).or_insert(0) += 1;
    }
    for (region, count) in targets.iter() {
        println!("  {} calls to {}", count, region);
    }
    // Show first 10 VRAM-targeted calls
    let mut idx = 0;
    for &(src, dst, cnt) in log {
        if dst >= 0x06000000 && dst < 0x06018000 {
            let fill = (cnt >> 24) & 1 != 0;
            let count = cnt & 0x1FFFFF;
            println!("  VRAM: src={:#X} dst={:#X} fill={} count={}", src, dst, fill, count);
            idx += 1;
            if idx >= 10 { break; }
        }
    }
}
