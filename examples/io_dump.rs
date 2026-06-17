use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

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
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    let dispcnt = gba.mem_mut().read_half(0x0400_0000);
    println!("DISPCNT={:#06X} = {:016b}", dispcnt, dispcnt);
    println!("  Mode: {}", dispcnt & 7);
    println!("  OBJ enable: {}", (dispcnt >> 6) & 1);
    println!("  BG0 enable: {}", (dispcnt >> 8) & 1);
    println!("  BG1 enable: {}", (dispcnt >> 9) & 1);
    println!("  BG2 enable: {}", (dispcnt >> 10) & 1);
    println!("  BG3 enable: {}", (dispcnt >> 11) & 1);
    println!("  WIN0 enable: {}", (dispcnt >> 13) & 1);
    println!("  WIN1 enable: {}", (dispcnt >> 14) & 1);
    println!("  OBJWIN enable: {}", (dispcnt >> 15) & 1);

    let io = gba.mem().io();

    println!("\n=== IO registers ===");
    for off in (0..0x56).step_by(2) {
        let val = u16::from_le_bytes([io[off], io[off + 1]]);
        if val != 0 {
            println!(
                "  [{:#06X}] = {:#06X} ({:016b})",
                0x04000000 + off,
                val,
                val
            );
        }
    }

    let win0h = u16::from_le_bytes([io[0x40], io[0x41]]);
    let win0v = u16::from_le_bytes([io[0x42], io[0x43]]);
    let win1h = u16::from_le_bytes([io[0x44], io[0x45]]);
    let win1v = u16::from_le_bytes([io[0x46], io[0x47]]);
    let winin = u16::from_le_bytes([io[0x48], io[0x49]]);
    let winout = u16::from_le_bytes([io[0x4A], io[0x4B]]);

    println!("\n=== Window registers ===");
    println!(
        "  WIN0H: left={} right={}",
        win0h & 0xFF,
        (win0h >> 8) & 0xFF
    );
    println!(
        "  WIN0V: top={} bottom={}",
        win0v & 0xFF,
        (win0v >> 8) & 0xFF
    );
    println!(
        "  WIN1H: left={} right={}",
        win1h & 0xFF,
        (win1h >> 8) & 0xFF
    );
    println!(
        "  WIN1V: top={} bottom={}",
        win1v & 0xFF,
        (win1v >> 8) & 0xFF
    );
    println!(
        "  WININ: WIN0={:05b} WIN1={:05b}",
        winin & 0x1F,
        (winin >> 8) & 0x1F
    );
    println!(
        "  WINOUT: outside={:05b} OBJWIN={:05b}",
        winout & 0x1F,
        (winout >> 8) & 0x1F
    );

    for bg in 0..4 {
        let bgcnt_off = 0x08 + bg * 2;
        let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
        let hofs = u16::from_le_bytes([io[0x10 + bg * 4], io[0x11 + bg * 4]]) & 0x1FF;
        let vofs = u16::from_le_bytes([io[0x12 + bg * 4], io[0x13 + bg * 4]]) & 0x1FF;
        let priority = bgcnt & 3;
        let tile_base = ((bgcnt >> 2) & 0x3) as u32 * 0x4000;
        let is_8bpp = (bgcnt & 0x80) != 0;
        let scr_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
        let size = (bgcnt >> 14) & 3;
        let enabled = (dispcnt >> (8 + bg)) & 1;
        println!("\n  BG{}: enabled={} priority={} tile_base={:#X} scr_base={:#X} {} size={} hofs={} vofs={}",
            bg, enabled, priority, tile_base, scr_base,
            if is_8bpp { "8bpp" } else { "4bpp" }, size, hofs, vofs);
    }

    println!("\n=== PPU snapshot state ===");
    let ppu = gba.ppu();
    println!("  DISPCNT from PPU: {:#06X}", ppu.get_dispcnt());
    println!("  Mode: {}", ppu.get_display_mode());
    for bg in 0..4 {
        println!(
            "  BG{} enabled: {} priority: {}",
            bg,
            ppu.is_bg_enabled(bg),
            ppu.get_bg_priority(bg)
        );
    }
}
