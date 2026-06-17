use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    gba.mem_mut().cpu_set_log_enabled = true;

    for _ in 0..200u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.mem_mut().cpu_set_log_enabled = false;

    let log = &gba.mem().cpu_set_log;
    println!("CpuSet calls: {}", log.len());

    let mut vram_copies = 0;
    for (src, dst, cnt) in log.iter() {
        let dst_region = if *dst >= 0x06000000 && *dst < 0x06018000 {
            "VRAM"
        } else if *dst >= 0x02000000 && *dst < 0x03000000 {
            "EWRAM"
        } else if *dst >= 0x03000000 && *dst < 0x04000000 {
            "IWRAM"
        } else {
            "???"
        };
        if dst_region == "VRAM" {
            vram_copies += 1;
        }
    }
    println!("CpuSet to VRAM: {}", vram_copies);

    for (src, dst, cnt) in log.iter() {
        if *dst >= 0x06000000 && *dst < 0x06018000 {
            let count = cnt & 0x1FFFFF;
            let fill = (cnt >> 24) & 1;
            let is_32 = (cnt >> 26) & 1;
            println!(
                "  CpuSet: {:#010X} -> {:#010X} cnt={}(fill={} 32bit={})",
                src, dst, count, fill, is_32
            );
        }
    }

    println!("\n=== Now let's look at ROM tile data ===");
    let rom = gba.mem().rom();
    println!(
        "ROM size: {} bytes ({:.1}MB)",
        rom.len(),
        rom.len() as f64 / 1048576.0
    );

    let vram = gba.mem().vram();

    println!("\n=== Checking if tile data is copied from ROM or EWRAM ===");

    let tile_0_data = &vram[0..32];
    let tile_50_data = &vram[50 * 32..50 * 32 + 32];
    let tile_100_data = &vram[100 * 32..100 * 32 + 32];

    for (name, data) in [
        ("tile_0", tile_0_data),
        ("tile_50", tile_50_data),
        ("tile_100", tile_100_data),
    ] {
        print!("{}: ", name);
        for b in data.iter().take(16) {
            print!("{:02X}", b);
        }
        println!();
    }

    println!("\n=== EWRAM analysis ===");
    let ewram = gba.mem().wram();
    let ewram_nonzero = ewram.iter().filter(|&&b| b != 0).count();
    println!(
        "EWRAM nonzero: {}/{} ({:.1}%)",
        ewram_nonzero,
        ewram.len(),
        ewram_nonzero as f64 / ewram.len() as f64 * 100.0
    );

    let mut ewram_regions = Vec::new();
    let mut in_region = false;
    let mut region_start = 0;
    for i in 0..ewram.len() {
        if ewram[i] != 0 && !in_region {
            in_region = true;
            region_start = i;
        } else if ewram[i] == 0 && in_region {
            ewram_regions.push((region_start, i));
            in_region = false;
        }
    }
    if in_region {
        ewram_regions.push((region_start, ewram.len()));
    }

    println!("EWRAM regions with data: {}", ewram_regions.len());
    for (start, end) in ewram_regions.iter().take(20) {
        let size = end - start;
        println!(
            "  {:#08X}..{:#08X} ({} bytes)",
            0x02000000 + start,
            0x02000000 + end,
            size
        );
    }

    println!("\n=== EWRAM tile data? ===");
    let mut tile_like = 0;
    for i in (0..ewram.len() - 32).step_by(32) {
        let has_data = (0..32).any(|j| ewram[i + j] != 0);
        if has_data {
            tile_like += 1;
        }
    }
    println!(
        "EWRAM 32-byte blocks with data: {}/{}",
        tile_like,
        ewram.len() / 32
    );
}
