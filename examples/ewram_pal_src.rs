use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..192u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check EWRAM at 0x0200871C (source of the overwriting DMA)
    let start = 0x0200871Cusize;
    let ewram = gba.mem.wram();

    println!("EWRAM at 0x0200871C (512 bytes):");
    let mut nonzero = 0;
    for i in 0..128 {
        let off = start - 0x02000000 + i * 4;
        let val = u32::from_le_bytes([ewram[off], ewram[off + 1], ewram[off + 2], ewram[off + 3]]);
        if val != 0 {
            nonzero += 1;
        }
        if i < 32 || val != 0 {
            println!(
                "  [{:03}] {:08X}: {:08X}{}",
                i,
                start + i * 4,
                val,
                if val != 0 { " *" } else { "" }
            );
        }
    }
    println!("Non-zero words: {}/128", nonzero);

    // Also check the ROM sources
    let rom = gba.mem.rom();
    println!("\nROM at 083E8E5C (32 bytes -> 050002E0):");
    let off1 = 0x083E8E5C - 0x08000000;
    for i in 0..8 {
        let val = u32::from_le_bytes([
            rom[off1 + i * 4],
            rom[off1 + i * 4 + 1],
            rom[off1 + i * 4 + 2],
            rom[off1 + i * 4 + 3],
        ]);
        println!("  [{:02}] {:08X}: {:08X}", i, 0x083E8E5C + i * 4, val);
    }

    println!("\nROM at 083E8CFC (32 bytes -> 05000280):");
    let off2 = 0x083E8CFC - 0x08000000;
    for i in 0..8 {
        let val = u32::from_le_bytes([
            rom[off2 + i * 4],
            rom[off2 + i * 4 + 1],
            rom[off2 + i * 4 + 2],
            rom[off2 + i * 4 + 3],
        ]);
        println!("  [{:02}] {:08X}: {:08X}", i, 0x083E8CFC + i * 4, val);
    }
}
