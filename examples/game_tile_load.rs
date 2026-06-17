use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.mem.vram_log_enabled = true;
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    // Run just a few frames
    for _ in 0..3 {
        gba.run_frame();
    }

    let log = &gba.mem.vram_write_log;
    println!("After 3 frames: {} VRAM writes", log.len());

    // Check what addresses were written
    let mut addr_set = std::collections::HashSet::new();
    for &(addr, pc, val) in log {
        let offset = (addr & 0x1FFFF) as u16;
        if offset >= 0x0000 && offset < 0x4000 && val != 0 {
            addr_set.insert(offset);
        }
    }
    let mut addrs: Vec<u16> = addr_set.into_iter().collect();
    addrs.sort();
    println!(
        "Non-zero writes to tile area (0x0000-0x3FFF): {} unique addresses",
        addrs.len()
    );
    for &a in addrs.iter().take(30) {
        println!("  {:#06X}", a);
    }
    if addrs.len() > 30 {
        println!("  ... and {} more", addrs.len() - 30);
    }

    // Max non-zero write address
    if let Some(&max) = addrs.last() {
        println!("Max non-zero tile write address: {:#06X}", max);
        println!(
            "That's tile {} row {}",
            max as u32 / 32,
            (max as u32 % 32) / 4
        );
    }

    // Check ALL writes (including zero) to 0x2000-0x3FFF
    let mut range2_writes = 0;
    for &(addr, _pc, _val) in log {
        let offset = addr & 0x1FFFF;
        if offset >= 0x2000 && offset < 0x4000 {
            range2_writes += 1;
        }
    }
    println!(
        "\nTotal writes to 0x2000-0x3FFF (including zeros): {}",
        range2_writes
    );

    // Check: what is the memory at the destination of the first DMA?
    // DMA3 is typically used for large transfers
    // Let's check DMA3 registers
    let io = gba.mem().io();
    let dma3sad = u32::from_le_bytes([io[0xD4], io[0xD5], io[0xD6], io[0xD7]]);
    let dma3dad = u32::from_le_bytes([io[0xD8], io[0xD9], io[0xDA], io[0xDB]]);
    let dma3cnt = u32::from_le_bytes([io[0xDC], io[0xDD], io[0xDE], io[0xDF]]);
    println!(
        "\nDMA3: SAD={:#010X} DAD={:#010X} CNT={:#010X}",
        dma3sad, dma3dad, dma3cnt
    );
}
