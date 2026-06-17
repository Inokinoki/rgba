use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..193 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("=== Frame 193 (display setup -> full display transition) ===");
    println!(
        "Before frame: VBLK={:08X} DISPCNT={:04X}",
        gba.mem.read_word(0x03007FF8),
        u16::from_le_bytes([gba.mem.io()[0], gba.mem.io()[1]])
    );

    for scanline in 0..228 {
        let before = gba.mem.read_word(0x03007FF8);
        gba.run_scanline();
        let after = gba.mem.read_word(0x03007FF8);
        if before != after {
            println!(
                "  Scanline {:3}: VBLK {:08X} -> {:08X} (delta={})",
                scanline,
                before,
                after,
                after.wrapping_sub(before)
            );
        }
    }

    println!(
        "After frame: VBLK={:08X} DISPCNT={:04X}",
        gba.mem.read_word(0x03007FF8),
        u16::from_le_bytes([gba.mem.io()[0], gba.mem.io()[1]])
    );

    gba.mem.iwram_write_log_enabled = true;
    gba.mem.iwram_write_log.clear();
    gba.mem.swi_log_enabled = true;
    gba.mem.swi_log.clear();

    println!("\n=== Frame 194 (full display) ===");
    for scanline in 0..228 {
        let before = gba.mem.read_word(0x03007FF8);
        gba.run_scanline();
        let after = gba.mem.read_word(0x03007FF8);
        if before != after {
            println!(
                "  Scanline {:3}: VBLK {:08X} -> {:08X}",
                scanline, before, after
            );
        }
    }

    println!("SWIs called: {:?}", gba.mem.swi_log);
    println!(
        "IWRAM writes to 0x7FF8 area: {} entries",
        gba.mem.iwram_write_log.len()
    );
    for (addr, pc, val) in &gba.mem.iwram_write_log {
        println!("  WRITE addr={:08X} pc={:08X} val={:02X}", addr, pc, val);
    }

    gba.mem.iwram_write_log_enabled = false;
    gba.mem.swi_log_enabled = false;
}
