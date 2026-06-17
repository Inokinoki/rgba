use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    let mut iwram_swi_calls = 0;
    let mut iwram_swi_nums: std::collections::BTreeMap<u32, u32> =
        std::collections::BTreeMap::new();

    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().swi_log.clear();

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let swi_log = &gba.mem().swi_log;
    println!("Total SWI calls: {}", swi_log.len());

    let mut counts = std::collections::BTreeMap::new();
    for &swi in swi_log {
        *counts.entry(swi).or_insert(0) += 1;
    }
    for (swi, count) in &counts {
        let name = match swi {
            0x00 => "SoftReset",
            0x01 => "RegisterRamReset",
            0x04 => "IntrWait",
            0x05 => "VBlankIntrWait",
            0x06 => "Div",
            0x08 => "Sqrt",
            0x0B => "CpuSet",
            0x0C => "CpuFastSet",
            0x10 => "LZ77UnCompReadNormal",
            0x11 => "LZ77UnCompReadByCallbackWrite16bit",
            _ => "Unknown",
        };
        println!("  SWI {:#04X} ({}): {} calls", swi, name, count);
    }

    println!("\nARM SWI count: {}", gba.mem().arm_swi_count);
    println!("THUMB SWI count: {}", gba.mem().thumb_swi_count);
}
