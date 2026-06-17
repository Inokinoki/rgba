use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..100 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("=== Our Emulator Frame 100 ===");
    println!("VBLK (03007FF8): 0x{:08X}", gba.mem.read_word(0x03007FF8));
    println!(
        "Handler (03007FFC): 0x{:08X}",
        gba.mem.read_word(0x03007FFC)
    );

    let io = gba.mem.io();
    println!("\nIO registers:");
    println!("  DISPCNT   = 0x{:04X}", u16::from_le_bytes([io[0], io[1]]));
    println!("  DISPSTAT  = 0x{:04X}", u16::from_le_bytes([io[4], io[5]]));
    println!("  VCOUNT    = 0x{:04X}", u16::from_le_bytes([io[6], io[7]]));
    println!("  IE        = 0x{:04X}", gba.mem.interrupt.ie.bits());
    println!(
        "  IF        = 0x{:04X}",
        u16::from_le_bytes([io[0x202], io[0x203]])
    );
    println!("  IME       = 0x{:04X}", gba.mem.interrupt.ime as u16);
    println!(
        "  TM0CNT_L  = 0x{:04X}",
        u16::from_le_bytes([io[0x100], io[0x101]])
    );
    println!(
        "  TM0CNT_H  = 0x{:04X}",
        u16::from_le_bytes([io[0x102], io[0x103]])
    );
    println!(
        "  TM1CNT_L  = 0x{:04X}",
        u16::from_le_bytes([io[0x104], io[0x105]])
    );
    println!(
        "  TM1CNT_H  = 0x{:04X}",
        u16::from_le_bytes([io[0x106], io[0x107]])
    );
    println!(
        "  TM2CNT_L  = 0x{:04X}",
        u16::from_le_bytes([io[0x108], io[0x109]])
    );
    println!(
        "  TM2CNT_H  = 0x{:04X}",
        u16::from_le_bytes([io[0x10A], io[0x10B]])
    );
    println!(
        "  TM3CNT_L  = 0x{:04X}",
        u16::from_le_bytes([io[0x10C], io[0x10D]])
    );
    println!(
        "  TM3CNT_H  = 0x{:04X}",
        u16::from_le_bytes([io[0x10E], io[0x10F]])
    );

    println!("\nGame state:");
    for (name, addr) in [
        ("State", 0x02000074u32),
        ("State2", 0x0200007Cu32),
        ("Counter", 0x02000064u32),
        ("Timer", 0x02000060u32),
        ("Ptr", 0x020000C0u32),
        ("State3", 0x02000068u32),
    ] {
        println!(
            "  {} [{:08X}] = 0x{:08X}",
            name,
            addr,
            gba.mem.read_word(addr)
        );
    }

    println!("\nEWRAM non-zero [0x02000000 - 0x02000100]:");
    let wram = gba.mem.wram();
    for off in (0..0x100).step_by(4) {
        let val = u32::from_le_bytes([wram[off], wram[off + 1], wram[off + 2], wram[off + 3]]);
        if val != 0 {
            println!("  [{:04X}] = 0x{:08X}", off, val);
        }
    }
}
