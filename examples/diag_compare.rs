use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run exactly 5 seconds worth of frames (300 frames at 60fps)
    // But since our emu might run at different speed, just run 300 frames
    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();

    let vram = gba.mem.vram();

    // Show BG0 screen entries at 0xC000, first 2 rows (raw values)
    eprintln!("Our BG0 screen entries at 0xC000, row 0:");
    for i in 0..32 {
        let addr = 0xC000 + i * 2;
        let entry = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
        let tile = entry & 0x3FF;
        let pal = (entry >> 12) & 0xF;
        eprintln!("  [{:2}] raw=0x{:04X} tile={} pal={}", i, entry, tile, pal);
    }

    eprintln!("\nOur BG0 screen entries at 0xC000, row 1:");
    for i in 0..32 {
        let addr = 0xC000 + (32 + i) * 2;
        let entry = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
        let tile = entry & 0x3FF;
        let pal = (entry >> 12) & 0xF;
        eprintln!("  [{:2}] raw=0x{:04X} tile={} pal={}", i, entry, tile, pal);
    }

    // Also check the 2nd screen block (for 64-wide BG)
    eprintln!("\nOur BG0 screen entries at 0xC800 (block 1), row 0:");
    for i in 0..32 {
        let addr = 0xC800 + i * 2;
        let entry = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
        let tile = entry & 0x3FF;
        let pal = (entry >> 12) & 0xF;
        eprintln!("  [{:2}] raw=0x{:04X} tile={} pal={}", i, entry, tile, pal);
    }

    // Compare tile data
    eprintln!("\nOur tile 277 data:");
    for i in 0..8 {
        let off = 277 * 32 + i * 4;
        let w = u32::from_le_bytes([vram[off], vram[off + 1], vram[off + 2], vram[off + 3]]);
        eprintln!("  word {}: 0x{:08X}", i, w);
    }

    eprintln!("\nOur tile 481 data:");
    for i in 0..8 {
        let off = 481 * 32 + i * 4;
        if off + 3 < vram.len() {
            let w = u32::from_le_bytes([vram[off], vram[off + 1], vram[off + 2], vram[off + 3]]);
            eprintln!("  word {}: 0x{:08X}", i, w);
        }
    }

    // Check IO registers
    eprintln!("\nOur IO registers:");
    eprintln!("DISPCNT: 0x{:04X}", gba.ppu.get_dispcnt());
    for bg in 0..4 {
        eprintln!("BG{}CNT: 0x{:04X}", bg, gba.ppu.get_bgcnt(bg));
    }
    eprintln!("BG0HOFS: {}", gba.ppu.get_bg_hofs(0));
    eprintln!("BG0VOFS: {}", gba.ppu.get_bg_vofs(0));
    eprintln!("BLDCNT: 0x{:04X}", gba.ppu.get_blend_control());
    eprintln!("BLDY: {}", gba.ppu.get_blend_brightness());
}
