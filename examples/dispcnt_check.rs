use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    // Boot to game
    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    // Press A 40 times to get past intro
    for _ in 0..40 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..5 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..55 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    // Check DISPCNT interpretation
    let dispcnt_raw = gba.mem_read_word(0x04000000) as u16;
    let dispcnt_ppu = gba.ppu().get_dispcnt();

    println!("=== DISPCNT Analysis ===");
    println!(
        "Raw value in memory: 0x{:04X} ({:016b})",
        dispcnt_raw, dispcnt_raw
    );
    println!(
        "PPU value (truncated): 0x{:04X} ({:016b})",
        dispcnt_ppu, dispcnt_ppu
    );
    println!(
        "Bits lost by truncation: 0x{:04X}",
        dispcnt_raw & !dispcnt_ppu
    );
    println!();

    // GBATEK interpretation
    println!("=== GBATEK Interpretation ===");
    println!("Mode: {}", dispcnt_raw & 0x7);
    println!("BG0 enable (bit 3): {}", (dispcnt_raw >> 3) & 1);
    println!("BG1 enable (bit 4): {}", (dispcnt_raw >> 4) & 1);
    println!("BG2 enable (bit 5): {}", (dispcnt_raw >> 5) & 1);
    println!("BG3 enable (bit 6): {}", (dispcnt_raw >> 6) & 1);
    println!("OBJ enable (bit 7): {}", (dispcnt_raw >> 7) & 1);
    println!("WIN0 enable (bit 8): {}", (dispcnt_raw >> 8) & 1);
    println!("WIN1 enable (bit 9): {}", (dispcnt_raw >> 9) & 1);
    println!("OBJ_WIN enable (bit 10): {}", (dispcnt_raw >> 10) & 1);
    println!("Special FX (bit 11): {}", (dispcnt_raw >> 11) & 1);
    println!("Frame select (bit 12): {}", (dispcnt_raw >> 12) & 1);
    println!("OBJ VRAM map (bit 15): {}", (dispcnt_raw >> 15) & 1);
    println!();

    // Emulator interpretation (current buggy code)
    println!("=== Emulator Interpretation (CURRENT - BUGGY) ===");
    for bg in 0..4 {
        println!(
            "is_bg_enabled({}): {} (checks bit {} of PPU DISPCNT)",
            bg,
            gba.ppu().is_bg_enabled(bg),
            8 + bg
        );
    }
    println!(
        "OBJ check (bit 12 of PPU DISPCNT): {}",
        (dispcnt_ppu >> 12) & 1
    );
    println!(
        "WIN0 check (bit 13 of PPU DISPCNT): {}",
        (dispcnt_ppu >> 13) & 1
    );
    println!(
        "WIN1 check (bit 14 of PPU DISPCNT): {}",
        (dispcnt_ppu >> 14) & 1
    );
    println!();

    // What SHOULD be enabled
    println!("=== What SHOULD be enabled ===");
    for bg in 0..4 {
        println!(
            "BG{}: {} (GBATEK bit {})",
            bg,
            (dispcnt_raw >> (3 + bg)) & 1,
            3 + bg
        );
    }
    println!("OBJ: {} (GBATEK bit 7)", (dispcnt_raw >> 7) & 1);
    println!();

    // BG register state
    for bg in 0..4 {
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let enabled = gba.ppu().is_bg_enabled(bg);
        let priority = bgcnt & 0x3;
        let tile_base = ((bgcnt >> 2) & 0x3) * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let is_8bpp = (bgcnt >> 7) & 1;
        let size = (bgcnt >> 14) & 3;
        println!(
            "BG{}: cnt=0x{:04X} enabled={} pri={} tile=0x{:04X} map=0x{:04X} {}bpp size={}",
            bg,
            bgcnt,
            enabled,
            priority,
            tile_base,
            map_base,
            if is_8bpp != 0 { 8 } else { 4 },
            size
        );
    }
}
