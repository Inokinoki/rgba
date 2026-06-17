use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    gba.input.press_key(rgba::KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input.release_key(rgba::KeyState::START);
    for _ in 0..180 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Press A through menus to reach gameplay
    for _ in 0..10 {
        gba.input.press_key(rgba::KeyState::A);
        for _ in 0..4 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input.release_key(rgba::KeyState::A);
        for _ in 0..16 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    gba.sync_ppu_full();

    let ppu = gba.ppu();
    let dc = ppu.get_dispcnt();
    let mode = dc & 0x7;
    println!("DISPCNT={:#06X} mode={}", dc, mode);

    // Check BG0 screen entries in VRAM (Mode 0: screen base at 0x0600C000)
    let vram = ppu.vram();
    println!("\n=== BG0 Screen Entries at VRAM 0x0600C000 (first 64) ===");
    for i in 0..64 {
        let offset = 0xC000 + i * 2;
        let val = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
        let tile = val & 0x3FF;
        let pal = (val >> 12) & 0xF;
        println!(
            "  [{:2}] {:08X}: {:04X} (tile={} pal={})",
            i,
            0x0600C000 + i * 2,
            val,
            tile,
            pal
        );
    }

    // Check IWRAM source
    let iwram = gba.mem().iwram();
    println!("\n=== IWRAM at 0x03006DD8 (first 64) ===");
    for i in 0..64 {
        let offset = 0x6DD8 + i * 2;
        let val = u16::from_le_bytes([iwram[offset], iwram[offset + 1]]);
        let tile = val & 0x3FF;
        let pal = (val >> 12) & 0xF;
        println!(
            "  [{:2}] {:08X}: {:04X} (tile={} pal={})",
            i,
            0x03006DD8 + i * 2,
            val,
            tile,
            pal
        );
    }

    // Compare
    let mut match_count = 0;
    let mut mismatch_count = 0;
    println!("\n=== Comparison ===");
    for i in 0..64 {
        let vram_val = u16::from_le_bytes([vram[0xC000 + i * 2], vram[0xC000 + i * 2 + 1]]);
        let iwram_val = u16::from_le_bytes([iwram[0x6DD8 + i * 2], iwram[0x6DD8 + i * 2 + 1]]);
        if vram_val == iwram_val {
            match_count += 1;
        } else {
            mismatch_count += 1;
            if mismatch_count <= 10 {
                println!(
                    "  MISMATCH [{:2}]: VRAM={:04X} IWRAM={:04X}",
                    i, vram_val, iwram_val
                );
            }
        }
    }
    println!("Match: {}, Mismatch: {}", match_count, mismatch_count);
}
