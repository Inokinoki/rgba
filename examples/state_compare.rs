use rgba::Gba;
use std::fs;

fn save_screenshot(fb: &[u32], png_path: &str) {
    let ppm_path = png_path.replace(".png", ".ppm");
    let mut bytes = b"P6\n240 160\n255\n".to_vec();
    for y in 0..160 {
        for x in 0..240 {
            let pixel = fb[y * 240 + x];
            bytes.push(((pixel >> 16) & 0xFF) as u8);
            bytes.push(((pixel >> 8) & 0xFF) as u8);
            bytes.push((pixel & 0xFF) as u8);
        }
    }
    fs::write(&ppm_path, &bytes).unwrap();
    std::process::Command::new("python3")
        .args([
            "-c",
            &format!(
                "from PIL import Image; Image.open('{}').save('{}')",
                ppm_path, png_path
            ),
        ])
        .output()
        .unwrap();
}

fn print_bg_state(gba: &Gba) {
    let ppu = gba.ppu();
    let dispcnt = ppu.get_dispcnt();
    println!("DISPCNT={:04X}", dispcnt);
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let priority = bgcnt & 3;
        let tile_base = ((bgcnt >> 2) & 3) * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        println!(
            "BG{}CNT={:04X} pri={} tile_base={:#X} map_base={:#X} size={} HOFS={:03X} VOFS={:03X}",
            bg, bgcnt, priority, tile_base, map_base, size, hofs, vofs
        );
    }
}

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..300u32 {
        gba.run_frame_parallel(&mut fb);
    }
    println!("=== Frame 300 ===");
    print_bg_state(&gba);

    for _ in 0..200u32 {
        gba.run_frame_parallel(&mut fb);
    }
    println!("\n=== Frame 500 ===");
    print_bg_state(&gba);

    gba.input_mut().press_key(rgba::KeyState::START);
    for _ in 0..100u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::START);
    for _ in 0..300u32 {
        gba.run_frame_parallel(&mut fb);
    }
    println!("\n=== After START (~frame 900) ===");
    print_bg_state(&gba);
    save_screenshot(&fb, "/tmp/our_after_start.png");

    gba.input_mut().press_key(rgba::KeyState::A);
    for _ in 0..100u32 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.input_mut().release_key(rgba::KeyState::A);
    for _ in 0..100u32 {
        gba.run_frame_parallel(&mut fb);
    }
    println!("\n=== After A (dialogue) ===");
    print_bg_state(&gba);
    save_screenshot(&fb, "/tmp/our_dialogue.png");

    let ppu = gba.ppu();
    let vram = ppu.vram();

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let map_base = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        println!(
            "\nBG{} map entries (map_base={:#X}, first 32):",
            bg, map_base
        );
        for i in 0..32usize {
            let addr = map_base + i * 2;
            if addr + 1 < vram.len() {
                let val = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
                let tile = val & 0x3FF;
                let pal = (val >> 12) & 0xF;
                println!("  entry[{}] = {:04X} (tile={} pal={})", i, val, tile, pal);
            }
        }
    }

    let io = gba.mem().io();
    let win0h = u16::from_le_bytes([io[0x40], io[0x41]]);
    let win0v = u16::from_le_bytes([io[0x42], io[0x43]]);
    let win1h = u16::from_le_bytes([io[0x44], io[0x45]]);
    let win1v = u16::from_le_bytes([io[0x46], io[0x47]]);
    let winin = u16::from_le_bytes([io[0x48], io[0x49]]);
    let winout = u16::from_le_bytes([io[0x4A], io[0x4B]]);
    println!(
        "\nWIN0H={:04X} WIN0V={:04X} WIN1H={:04X} WIN1V={:04X} WININ={:04X} WINOUT={:04X}",
        win0h, win0v, win1h, win1v, winin, winout
    );
    let bldcnt = u16::from_le_bytes([io[0x50], io[0x51]]);
    let bldalpha = u16::from_le_bytes([io[0x52], io[0x53]]);
    println!("BLDCNT={:04X} BLDALPHA={:04X}", bldcnt, bldalpha);

    println!("\nPalette bank 11 (entries 176-191):");
    for i in 176..192u16 {
        let color = gba.get_palette_color(0, i);
        if color != 0 {
            let r = color & 0x1F;
            let g = (color >> 5) & 0x1F;
            let b = (color >> 10) & 0x1F;
            println!("  pal[{}] = {:04X} (r={} g={} b={})", i, color, r, g, b);
        }
    }

    // Also dump mem.vram() directly (not PPU copy)
    let mem_vram = gba.mem().vram();
    println!("\nMem VRAM at BG0 map base (0xC000), first 32:");
    for i in 0..32usize {
        let addr = 0xC000 + i * 2;
        if addr + 1 < mem_vram.len() {
            let val = u16::from_le_bytes([mem_vram[addr], mem_vram[addr + 1]]);
            let tile = val & 0x3FF;
            let pal = (val >> 12) & 0xF;
            println!("  mem[{}] = {:04X} (tile={} pal={})", i, val, tile, pal);
        }
    }
}
