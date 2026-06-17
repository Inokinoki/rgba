use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().cpu_set_log_enabled = true;
    gba.mem_mut().cpu_set_log.clear();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().cpu_set_log;
    println!("CpuSet calls: {}", log.len());
    for (i, (src, dst, cnt)) in log.iter().enumerate() {
        let fill = (cnt >> 24) & 1 != 0;
        let count = cnt & 0x1FFFFF;
        let is_32 = (cnt >> 26) & 1 != 0;
        let is_vram = *dst >= 0x0600_0000 && *dst < 0x0602_0000;
        let is_rom = *src >= 0x0800_0000;
        println!(
            "  CpuSet {}: src={:#010X} dst={:#010X} count={} fill={} is32={} {}{}",
            i,
            src,
            dst,
            count,
            fill,
            is_32,
            if is_vram { "[→VRAM]" } else { "" },
            if is_rom { "[ROM→]" } else { "" }
        );
    }
}
