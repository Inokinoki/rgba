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

    // Check BG3 tile map entries for the VISIBLE area
    // BG3: cnt=0x5E40, map_base=0xF000, tile_base=0, size=1 (64x32), hofs=224, vofs=0
    // At screen (3,3): map_x=(3+224)%512=227, tile_x=28
    // BG3 has same scroll as BG0 (h=224, v=0)

    let ppu = &gba.ppu;

    // Check all BGs' tile map entries at tile column 28, row 0
    for bg in 0..4 {
        let cnt = ppu.get_bgcnt(bg);
        let map_base = ppu.get_bg_map_base(bg) as usize;
        let hofs = ppu.get_bg_hofs(bg);
        let vofs = ppu.get_bg_vofs(bg);
        let map_x = (3u32 + hofs as u32) % 512;
        let map_y = (3u32 + vofs as u32) % 256;
        let tile_x = (map_x / 8) as u16;
        let tile_y = (map_y / 8) as u16;
        let bg_size = (cnt >> 14) & 0x3;

        let entry = ppu.get_screen_entry(map_base, tile_x, tile_y, bg_size, 64, 32);
        let tile_num = entry & 0x3FF;
        let pal = (entry >> 12) & 0xF;
        let tile_base = ppu.get_bg_tile_base(bg) as usize;

        // Get the actual pixel
        let px = gba.get_bg_pixel(ppu, 0, bg, 3, 3);

        println!(
            "BG{}: cnt=0x{:04X} map=0x{:X} tile_base=0x{:X} entry=0x{:04X} tile={}:{} px={:?}",
            bg, cnt, map_base, tile_base, entry, tile_num, pal, px
        );
    }

    // Now: what if the PPU snapshot-based renderer in render_tile_pixel_composited
    // works differently from get_pixel_tile_mode?
    // render_tile_pixel_composited sorts by priority and takes the first non-transparent
    // get_pixel_tile_mode iterates bg 0..3 and skips if priority >= first_priority

    // Both should give the same result for BG0.
    // Let me check if get_pixel_tile_mode is actually returning the blend-applied value
    // by checking what blend_up(0x7E80, 13) is:
    let c = 0x7E80u16;
    let ey = 13u32;
    let r = ((c & 0x1F) as u32 + ((31 - (c & 0x1F) as u32) * ey) / 16);
    let g = (((c >> 5) & 0x1F) as u32 + ((31 - ((c >> 5) & 0x1F) as u32) * ey) / 16);
    let b = (((c >> 10) & 0x1F) as u32 + ((31 - ((c >> 10) & 0x1F) as u32) * ey) / 16);
    let blended = r.min(31) as u16 | ((g.min(31) as u16) << 5) | ((b.min(31) as u16) << 10);
    println!(
        "\nblend_up(0x7E80, 13) = 0x{:04X} (r={} g={} b={})",
        blended, r, g, b
    );

    // And backdrop:
    let c2 = 0x03E0u16;
    let r2 = ((c2 & 0x1F) as u32 + ((31 - (c2 & 0x1F) as u32) * ey) / 16);
    let g2 = (((c2 >> 5) & 0x1F) as u32 + ((31 - ((c2 >> 5) & 0x1F) as u32) * ey) / 16);
    let b2 = (((c2 >> 10) & 0x1F) as u32 + ((31 - ((c2 >> 10) & 0x1F) as u32) * ey) / 16);
    let blended2 = r2.min(31) as u16 | ((g2.min(31) as u16) << 5) | ((b2.min(31) as u16) << 10);
    println!(
        "blend_up(0x03E0, 13) = 0x{:04X} (r2={} g2={} b2={})",
        blended2, r2, g2, b2
    );

    let result = gba.get_pixel_tile_mode(3, 3);
    println!("\nget_pixel_tile_mode(3,3) = 0x{:04X}", result);
    println!(
        "Match BG0 blend: {}  Match backdrop blend: {}",
        result == blended,
        result == blended2
    );
}
