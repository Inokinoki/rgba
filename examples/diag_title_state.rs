use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let frames = 200u32;
    let mut gba = Gba::new();
    gba.load_rom(rom_data);
    for _ in 0..frames {
        gba.run_frame();
    }

    // Check PPU state
    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let dispcnt = ppu.get_dispcnt();
    let mode = ppu.get_display_mode();
    eprintln!("DISPCNT=0x{:04X} mode={}", dispcnt, mode);
    eprintln!(
        "BG0 enabled: {}, BG1: {}, BG2: {}, BG3: {}, OBJ: {}",
        ppu.is_bg_enabled(0),
        ppu.is_bg_enabled(1),
        ppu.is_bg_enabled(2),
        ppu.is_bg_enabled(3),
        dispcnt & (1 << 12) != 0
    );

    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        let prio = ppu.get_bg_priority(bg);
        let tile_base = (bgcnt & 0x3) as u32 * 0x4000;
        let map_base = ((bgcnt >> 8) & 0x1F) as u32 * 0x800;
        let size = (bgcnt >> 14) & 0x3;
        eprintln!(
            "BG{}: bgcnt=0x{:04X} prio={} tile_base=0x{:05X} map_base=0x{:05X} size={}",
            bg, bgcnt, prio, tile_base, map_base, size
        );
        eprintln!(
            "  hofs={} vofs={}",
            ppu.get_bg_hofs(bg),
            ppu.get_bg_vofs(bg)
        );
    }

    eprintln!("\nVCount={}", ppu.get_vcount());
    eprintln!("DISPSTAT=0x{:04X}", ppu.get_dispstat());

    // Check what DMA3 state is
    drop(ppu);
    let io = gba.mem().io();
    let dma3_base = 0xB0 + 3 * 12;
    let dma3_cnt = u16::from_le_bytes([io[dma3_base + 10], io[dma3_base + 11]]);
    let dma3_src = u32::from_le_bytes([
        io[dma3_base],
        io[dma3_base + 1],
        io[dma3_base + 2],
        io[dma3_base + 3],
    ]);
    let dma3_dst = u32::from_le_bytes([
        io[dma3_base + 4],
        io[dma3_base + 5],
        io[dma3_base + 6],
        io[dma3_base + 7],
    ]);
    let dma3_cnt_val = u16::from_le_bytes([io[dma3_base + 8], io[dma3_base + 9]]);
    eprintln!(
        "\nDMA3: src=0x{:08X} dst=0x{:08X} count={} control=0x{:04X}",
        dma3_src, dma3_dst, dma3_cnt_val, dma3_cnt
    );
    eprintln!(
        "  enabled={} trigger={:?} repeat={}",
        dma3_cnt & 0x8000 != 0,
        match (dma3_cnt >> 12) & 3 {
            0 => "Imm",
            1 => "VBlank",
            2 => "HBlank",
            _ => "Special",
        },
        dma3_cnt & 0x0200 != 0
    );

    // Check all DMA channels
    for i in 0..4 {
        let base = 0xB0 + i * 12;
        let cnt = u16::from_le_bytes([io[base + 10], io[base + 11]]);
        if cnt != 0 {
            let src = u32::from_le_bytes([io[base], io[base + 1], io[base + 2], io[base + 3]]);
            let dst = u32::from_le_bytes([io[base + 4], io[base + 5], io[base + 6], io[base + 7]]);
            eprintln!(
                "DMA{}: src=0x{:08X} dst=0x{:08X} cnt=0x{:04X}",
                i, src, dst, cnt
            );
        }
    }

    // Check EWRAM data where DMA3 would copy from
    // If src=0x02008D88, check what data is there
    if dma3_src >= 0x02000000 && dma3_src < 0x02040000 {
        eprintln!("\nEWRAM at DMA3 src (0x{:08X}):", dma3_src);
        let ewram_off = (dma3_src - 0x02000000) as usize;
        let ewram = gba.mem().wram();
        for i in 0..64.min(ewram.len() - ewram_off) {
            if i % 16 == 0 {
                eprint!("\n  {:04X}: ", ewram_off + i);
            }
            eprint!("{:02X} ", ewram[ewram_off + i]);
        }
        eprintln!();
    }

    // Check OAM data in memory more carefully
    let oam = gba.mem().oam();
    eprintln!("\nFull OAM dump (first 128 bytes = 16 sprites):");
    for s in 0..16 {
        let off = s * 8;
        let a0 = u16::from_le_bytes([oam[off], oam[off + 1]]);
        let a1 = u16::from_le_bytes([oam[off + 2], oam[off + 3]]);
        let a2 = u16::from_le_bytes([oam[off + 4], oam[off + 5]]);
        let a3 = u16::from_le_bytes([oam[off + 6], oam[off + 7]]);
        if a0 != 0 || a1 != 0 || a2 != 0 {
            let y = a0 & 0xFF;
            let rot_scale = (a0 >> 8) & 1;
            let double_size = (a0 >> 9) & 1;
            let obj_mode = (a0 >> 10) & 3;
            let mosaic = (a0 >> 12) & 1;
            let color_mode = (a0 >> 13) & 1;
            let shape = (a0 >> 14) & 3;
            let x = a1 & 0x1FF;
            let size = (a1 >> 14) & 3;
            let tile_num = a2 & 0x3FF;
            let prio = (a2 >> 10) & 3;
            let pal = (a2 >> 12) & 0xF;
            eprintln!("  Spr{:3}: a0=0x{:04X} a1=0x{:04X} a2=0x{:04X} a3=0x{:04X} | y={} x={} tile={} prio={} pal={} shape={} size={} rot={} dbl={} mode={} col={}",
                s, a0, a1, a2, a3, y, x, tile_num, prio, pal, shape, size, rot_scale, double_size, obj_mode, color_mode);
        }
    }

    // Check which sprites are at y < 160 (potentially on screen)
    let mut on_screen = 0;
    for s in 0..128 {
        let off = s * 8;
        let a0 = u16::from_le_bytes([oam[off], oam[off + 1]]);
        let y = a0 & 0xFF;
        let mode = (a0 >> 14) & 3;
        if mode != 0b10 && y < 160 && a0 != 0 {
            on_screen += 1;
        }
    }
    eprintln!(
        "\nSprites potentially on-screen (y<160, enabled): {}",
        on_screen
    );

    // Check palette
    let pal_obj = &gba.mem().palette()[0x200..];
    let mut nonzero_pal = 0;
    for i in 0..256 {
        let c = u16::from_le_bytes([pal_obj[i * 2], pal_obj[i * 2 + 1]]);
        if c != 0 {
            nonzero_pal += 1;
        }
    }
    eprintln!("OBJ palette nonzero: {}/256", nonzero_pal);

    // Render screenshot
    drop(oam);
    let mut pixels = Vec::new();
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = gba.get_pixel_tile_mode(x, y);
            let r = ((c & 0x1F) as u32 * 255 / 31) << 16;
            let g = (((c >> 5) & 0x1F) as u32 * 255 / 31) << 8;
            let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
            pixels.push(r | g | b);
        }
    }

    // Check unique colors
    let mut unique_colors = std::collections::HashSet::new();
    for y in 0..160u16 {
        for x in 0..240u16 {
            unique_colors.insert(gba.get_pixel_tile_mode(x, y));
        }
    }
    eprintln!("\nUnique colors on screen: {}", unique_colors.len());
}
