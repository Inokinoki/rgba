use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data);

    for frame in 0..6u32 {
        let int_ctrl = &gba.mem().interrupt;
        let ie_actual = int_ctrl.ie.bits();
        let ime_actual = int_ctrl.ime;
        let if_actual = int_ctrl.if_raw.bits();
        eprintln!(
            "F{}: PC=0x{:08X} IE=0x{:04X} IME={} IF=0x{:04X} halt={} DC=0x{:04X}",
            frame,
            gba.cpu_pc(),
            ie_actual,
            ime_actual,
            if_actual,
            gba.cpu().is_halted(),
            gba.ppu().get_dispcnt()
        );

        // Also check IO array
        let io = gba.mem().io();
        let ie_io = u16::from_le_bytes([io[0x200], io[0x201]]);
        let ime_io = u16::from_le_bytes([io[0x208], io[0x209]]);
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        eprintln!(
            "  IO array: IE=0x{:04X} IME={} DC=0x{:04X}",
            ie_io, ime_io, dispcnt
        );
        eprintln!(
            "  Match: IE={} IME={}",
            ie_actual == ie_io,
            ime_actual as u16 == ime_io
        );

        drop(io);
        drop(int_ctrl);
        gba.run_frame();
    }

    // Continue for more frames
    for frame in 6..210u32 {
        gba.run_frame();
        if frame % 20 == 0 || frame < 10 {
            let int_ctrl = &gba.mem().interrupt;
            let ie_actual = int_ctrl.ie.bits();
            let ime_actual = int_ctrl.ime;
            let if_actual = int_ctrl.if_raw.bits();
            eprintln!(
                "F{}: PC=0x{:08X} IE=0x{:04X} IME={} IF=0x{:04X} halt={} DC=0x{:04X}",
                frame,
                gba.cpu_pc(),
                ie_actual,
                ime_actual,
                if_actual,
                gba.cpu().is_halted(),
                gba.ppu().get_dispcnt()
            );
        }
    }

    // Final state
    let int_ctrl = &gba.mem().interrupt;
    eprintln!("\nFinal state:");
    eprintln!(
        "  IE=0x{:04X} IME={} IF=0x{:04X}",
        int_ctrl.ie.bits(),
        int_ctrl.ime,
        int_ctrl.if_raw.bits()
    );
    eprintln!("  halt={}", gba.cpu().is_halted());
    eprintln!("  PC=0x{:08X}", gba.cpu_pc());

    // Render and check
    gba.sync_ppu_full();
    gba.sync_ppu();

    // Check OAM from memory
    let oam = gba.mem().oam();
    let mut nonzero = 0;
    for s in 0..128 {
        let off = s * 8;
        let a0 = u16::from_le_bytes([oam[off], oam[off + 1]]);
        if a0 != 0 {
            nonzero += 1;
        }
    }
    eprintln!("  OAM nonzero sprites: {}/128", nonzero);

    // Check unique colors
    let mut sprite_pixels = 0u32;
    let mut unique_colors = std::collections::HashSet::new();
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = gba.get_pixel_tile_mode(x, y);
            unique_colors.insert(c);
            let ppu = gba.ppu();
            if gba.get_sprite_pixel(ppu, x, y).is_some() {
                sprite_pixels += 1;
            }
        }
    }
    eprintln!("  Unique colors: {}", unique_colors.len());
    eprintln!("  Sprite pixels: {}", sprite_pixels);
}
