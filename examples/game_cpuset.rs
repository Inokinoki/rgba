use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.cpu_set_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for _ in 0..600 {
        gba.run_frame();
    }

    let log = &gba.mem.cpu_set_log;
    println!("CpuSet calls: {}", log.len());

    for (i, &(src, dst, cnt)) in log.iter().enumerate() {
        let fill = (cnt >> 24) & 1 != 0;
        let count = cnt & 0x1FFFFF;
        let is_32 = (cnt >> 26) & 1 != 0;
        let total_bytes = if is_32 { count * 4 } else { count * 2 };
        let dst_name = if dst >= 0x0600_0000 && dst < 0x0601_8000 {
            let off = dst - 0x0600_0000;
            if off < 0xC000 {
                format!("VRAM tile {} ({:#06X})", off / 32, off)
            } else if off < 0x10000 {
                format!("VRAM scrbase {:#06X}", off)
            } else {
                format!("VRAM OBJ {:#06X}", off)
            }
        } else if dst >= 0x0500_0000 && dst < 0x0500_0400 {
            "PALETTE".to_string()
        } else if dst >= 0x0200_0000 && dst < 0x0204_0000 {
            format!("EWRAM {:#010X}", dst)
        } else if dst >= 0x0300_0000 && dst < 0x0300_8000 {
            format!("IWRAM {:#010X}", dst)
        } else {
            format!("{:#010X}", dst)
        };
        let src_name = if src >= 0x0800_0000 && src < 0x0E00_0000 {
            format!("ROM {:#010X}", src)
        } else if src >= 0x0200_0000 && src < 0x0204_0000 {
            format!("EWRAM {:#010X}", src)
        } else if src >= 0x0300_0000 && src < 0x0300_8000 {
            format!("IWRAM {:#010X}", src)
        } else {
            format!("{:#010X}", src)
        };
        println!(
            "[{}] {}→{} count={} {}{} total={}",
            i,
            src_name,
            dst_name,
            count,
            if is_32 { "32bit " } else { "16bit " },
            if fill { "FILL" } else { "COPY" },
            total_bytes
        );
    }
}
