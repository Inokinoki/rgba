use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    // Test blend_brightness_up manually
    fn blend_up(c: u16, ey: u32) -> u16 {
        let r = ((c & 0x1F) as u32 + ((31 - (c & 0x1F) as u32) * ey) / 16);
        let g = (((c >> 5) & 0x1F) as u32 + ((31 - ((c >> 5) & 0x1F) as u32) * ey) / 16);
        let b = (((c >> 10) & 0x1F) as u32 + ((31 - ((c >> 10) & 0x1F) as u32) * ey) / 16);
        r.min(31) as u16 | ((g.min(31) as u16) << 5) | ((b.min(31) as u16) << 10)
    }

    // Test with BG0 color and backdrop
    let bg0_color = 0x7E80u16;
    let backdrop = 0x03E0u16;

    println!(
        "blend_up(0x{:04X}, 13) = 0x{:04X}",
        bg0_color,
        blend_up(bg0_color, 13)
    );
    println!(
        "blend_up(0x{:04X}, 13) = 0x{:04X}",
        backdrop,
        blend_up(backdrop, 13)
    );

    // Test actual get_pixel_tile_mode
    let result = gba.get_pixel_tile_mode(3, 3);
    println!("\nget_pixel_tile_mode(3,3) = 0x{:04X}", result);

    // Check: maybe the function uses the WRONG layer for blending?
    // Let me check what second_color is...
    // In the composited rendering, second_color should be 0 (no second layer)
    // But maybe there's an issue where the backdrop is used as second_color?

    // Wait - let me re-read get_pixel_tile_mode more carefully
    // Line 1067: for bg in 0..4
    // BG0 (pri=3) returns Some(0x7E80) -> first_color=0x7E80, first_priority=3
    // BG1 (pri=2): 2 < 3 -> returns None
    // BG2 (pri=1): 1 < 3 -> returns None
    // BG3 (pri=0): 0 < 3 -> returns None
    // first_color = 0x7E80, first_type = BG(0), first_priority = 3

    // Then check sprite... probably None
    // Then apply_pixel_blending(0x7E80, 0, BG(0), win_vis)
    // BLDCNT mode=2, BG0 is first target -> blend_brightness_up(0x7E80, 13)
    // = 0x7E19

    // But actual result is 0x7F99 = blend_up(0x03E0, 13)
    // This means the function is using the BACKDROP color instead of BG0!

    // Maybe the issue is that BG3 (priority 0) is checked first and returns
    // a non-None value even though it should be transparent?
    // Or maybe there's a different rendering path being used.

    // Let me check if there's a different code path for mode 0
    println!("\n=== Direct function call test ===");
    let ppu = &gba.ppu;
    let bg0_px = gba.get_bg_pixel(ppu, 0, 0, 3, 3);
    println!("get_bg_pixel(mode=0, bg=0, 3, 3) = {:?}", bg0_px);

    // Check what the PPU's snapshot renderer would produce
    let ppu_vram = gba.ppu.vram();
    let ppu_palette = gba.ppu.palette();
    let snapshot = gba.ppu.snapshot();
    let snap_color = rgba::Ppu::render_tile_pixel_composited(&snapshot, 3, 3, &ppu_palette);
    println!("render_tile_pixel_composited(3, 3) = 0x{:04X}", snap_color);
}
