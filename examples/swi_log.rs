use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().swi_log_enabled = true;
    gba.mem_mut().swi_log.clear();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let swi_log = &gba.mem().swi_log;
    println!("Total SWI calls: {}", swi_log.len());

    let mut counts = std::collections::BTreeMap::new();
    for &swi in swi_log {
        *counts.entry(swi).or_insert(0) += 1;
    }

    println!("\nSWI call counts:");
    for (swi, count) in &counts {
        let name = match swi {
            0x00 => "SoftReset",
            0x01 => "RegisterRamReset",
            0x02 => "Halt",
            0x03 => "Stop",
            0x04 => "IntrWait",
            0x05 => "VBlankIntrWait",
            0x06 => "Div",
            0x07 => "DivArm",
            0x08 => "Sqrt",
            0x0A => "ArcTan",
            0x0B => "CpuSet",
            0x0C => "CpuFastSet",
            0x0D => "GetBiosChecksum",
            0x0E => "BgAffineSet",
            0x0F => "ObjAffineSet",
            0x10 => "LZ77UnCompReadNormal",
            0x11 => "LZ77UnCompReadByCallbackWrite16bit",
            0x12 => "HuffUnCompReadNormal",
            0x13 => "HuffUnCompReadByCallback",
            0x14 => "RLUnCompReadNormal",
            0x15 => "RLUnCompReadByCallback",
            0x16 => "Diff8bitUnFilter",
            0x17 => "Diff16bitUnFilter",
            0x18 => "SoundBias",
            0x19 => "SoundDriverInit",
            0x1A => "SoundDriverMode",
            0x1B => "SoundDriverMain",
            0x1C => "SoundDriverVSync",
            _ => "Unknown",
        };
        println!("  SWI {:#04X} ({}): {} calls", swi, name, count);
    }
}
