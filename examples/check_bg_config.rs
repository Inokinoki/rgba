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
    let io = gba.mem().io();

    let dc = u16::from_le_bytes([io[0], io[1]]);
    println!(
        "DISPCNT={:#06X} mode={} BG_enable={:04b} OBJ={}",
        dc,
        dc & 7,
        (dc >> 8) & 0xF,
        (dc >> 12) & 1
    );

    for bg in 0..4 {
        let bgcnt_off = 0x08 + bg * 2;
        let bgcnt = u16::from_le_bytes([io[bgcnt_off], io[bgcnt_off + 1]]);
        let bgh = u16::from_le_bytes([io[0x10 + bg * 4], io[0x11 + bg * 4]]);
        let bgv = u16::from_le_bytes([io[0x12 + bg * 4], io[0x13 + bg * 4]]);
        let screen_base = (bgcnt >> 8) & 0x1F;
        let char_base = (bgcnt >> 2) & 3;
        let size = bgcnt & 3;
        let mosaic = (bgcnt >> 6) & 1;
        let palette_mode = (bgcnt >> 7) & 1;
        let priority = bgcnt & 3;
        println!("BG{}: BGCNT={:#06X} pri={} screen_base={:#04X} ({:#010X}) char_base={:#04X} ({:#010X}) size={} mosaic={} pal_mode={} BGHOFS={} BGVOFS={}",
            bg, bgcnt, priority, screen_base, 0x06000000 + (screen_base as u32) * 0x400,
            char_base, 0x06000000 + (char_base as u32) * 0x4000,
            size, mosaic, palette_mode, bgh, bgv);
    }

    // Check which VRAM areas have non-0xFF screen entries
    let vram = ppu.vram();
    for region in 0..16 {
        let base = region * 0x400;
        let mut non_ff = 0;
        let mut total = 0;
        for i in 0..0x200 {
            let val = u16::from_le_bytes([vram[base + i * 2], vram[base + i * 2 + 1]]);
            total += 1;
            if val != 0xFFFF && val != 0 {
                non_ff += 1;
            }
        }
        if non_ff > 0 {
            println!(
                "\nScreen base {:#04X} ({:#010X}): {}/{} non-zero/non-FF entries",
                region,
                0x06000000 + base,
                non_ff,
                total
            );
            // Show first few
            for i in 0..4 {
                let val = u16::from_le_bytes([vram[base + i * 2], vram[base + i * 2 + 1]]);
                print!("  [{:2}] {:04X}", i, val);
            }
            println!();
        }
    }
}
