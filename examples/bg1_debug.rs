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

    let mode = gba.ppu.get_display_mode();
    println!("PPU display mode: {}", mode);

    // Check BG1 pixel at (3,3) with mode=0
    for bg in 0..4 {
        let px = gba.get_bg_pixel(&gba.ppu, mode, bg, 3, 3);
        println!("get_bg_pixel(mode={}, bg={}, 3, 3) = {:?}", mode, bg, px);
    }

    // Check what BG1's tile map returns
    let bg1cnt = gba.ppu.get_bgcnt(1);
    let bg1_map_base = gba.ppu.get_bg_map_base(1) as usize;
    let bg1_tile_base = gba.ppu.get_bg_tile_base(1) as usize;
    let bg1_size = (bg1cnt >> 14) & 0x3;
    let hofs1 = gba.ppu.get_bg_hofs(1);
    println!(
        "\nBG1: cnt=0x{:04X} map=0x{:X} tile=0x{:X} size={} hofs={}",
        bg1cnt, bg1_map_base, bg1_tile_base, bg1_size, hofs1
    );

    let entry = gba
        .ppu
        .get_screen_entry(bg1_map_base, 28, 0, bg1_size, 512 / 8, 256 / 8);
    println!("BG1 screen entry at tile (28,0): 0x{:04X}", entry);

    // Check tile 0x3FF in PPU's VRAM
    let ppu_vram = gba.ppu.vram();
    let tile_base_addr = 0x3FF * 32;
    let mut all_zero = true;
    for i in 0..32 {
        if ppu_vram[tile_base_addr + i] != 0 {
            all_zero = false;
            break;
        }
    }
    println!("Tile 0x3FF all zero in PPU VRAM: {}", all_zero);

    // Actually check what tile pixel BG1 returns
    // BG1 hscroll = 224, so at screen (3,3): map_x = 227, tile_x = 28
    let bg1_entry = gba
        .ppu
        .get_screen_entry(bg1_map_base, 28, 0, bg1_size, 64, 32);
    let tile_num = bg1_entry & 0x3FF;
    println!(
        "BG1 tile 28,0: entry=0x{:04X} tile_num={}",
        bg1_entry, tile_num
    );

    // Read the tile pixel at (3,3) in this tile
    if tile_num < 1024 {
        let off = tile_num as usize * 32 + 3 * 4 + 3 / 2;
        let byte = ppu_vram[off];
        let ci = if 3 % 2 == 0 {
            byte & 0xF
        } else {
            (byte >> 4) & 0xF
        };
        println!("BG1 tile pixel (3,3): byte=0x{:02X} color_idx={}", byte, ci);
    }

    // Check mosaic
    let (mx, my) = gba.ppu.apply_bg_mosaic(227u16, 3u16);
    println!("\nMosaic applied: ({}, {}) -> ({}, {})", 227, 3, mx, my);
}
