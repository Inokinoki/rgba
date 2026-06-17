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

    // After run_frame_parallel, sync_ppu_full has been called
    // Check if PPU's VRAM copy matches Memory's VRAM
    let mem_vram = gba.mem.vram();
    let ppu_vram = gba.ppu.vram();

    let mut mismatches = 0;
    for i in 0..0x10000 {
        if mem_vram[i] != ppu_vram[i] {
            mismatches += 1;
            if mismatches <= 5 {
                println!(
                    "VRAM mismatch at 0x{:05X}: mem={:02X} ppu={:02X}",
                    i, mem_vram[i], ppu_vram[i]
                );
            }
        }
    }
    println!("Total VRAM mismatches: {}/65536", mismatches);

    // Check BG0 tile map entry at tile (28,0) in PPU's VRAM
    // screen_base = 0xC000, block_x = 28/32 = 0, local_x = 28
    let entry_off = 0xC000 + (0 * 32 + 28) * 2;
    let ppu_entry = u16::from_le_bytes([ppu_vram[entry_off], ppu_vram[entry_off + 1]]);
    let mem_entry = u16::from_le_bytes([mem_vram[entry_off], mem_vram[entry_off + 1]]);
    println!(
        "\nTile (28,0) entry: PPU=0x{:04X} MEM=0x{:04X}",
        ppu_entry, mem_entry
    );

    // Check PPU's BG0CNT and scroll
    let ppu_bg0cnt = gba.ppu.get_bgcnt(0);
    let ppu_bg0hofs = gba.ppu.get_bg_hofs(0);
    let ppu_bg0vofs = gba.ppu.get_bg_vofs(0);
    println!(
        "PPU BG0: cnt=0x{:04X} hofs={} vofs={}",
        ppu_bg0cnt, ppu_bg0hofs, ppu_bg0vofs
    );

    // Now try calling get_pixel_tile_mode at (3,3)
    let color = gba.get_pixel_tile_mode(3, 3);
    let r = ((color & 0x1F) as u32 * 255 / 31) << 16;
    let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
    let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
    println!(
        "\nget_pixel_tile_mode(3,3) = 0x{:04X} -> RGB 0x{:06X}",
        color,
        r | g | b
    );

    // Check what get_bg_pixel returns for BG0 at (3,3)
    let bg0_result = gba.get_bg_pixel(&gba.ppu, 0, 0, 3, 3);
    println!("get_bg_pixel(BG0, 3,3) = {:?}", bg0_result);

    // Check get_screen_entry manually
    let screen_base = gba.ppu.get_bg_map_base(0) as usize;
    let bg0cnt = gba.ppu.get_bgcnt(0);
    let bg_size = (bg0cnt >> 14) & 0x3;
    let hofs = gba.ppu.get_bg_hofs(0);
    let vofs = gba.ppu.get_bg_vofs(0);
    let map_x = (3u32 + hofs as u32) % 512;
    let map_y = (3u32 + vofs as u32) % 256;
    let tile_x = (map_x / 8) as u16;
    let tile_y = (map_y / 8) as u16;
    let entry = gba
        .ppu
        .get_screen_entry(screen_base, tile_x, tile_y, bg_size, 512 / 8, 256 / 8);
    println!(
        "\nscreen_base=0x{:X} hofs={} vofs={} map=({},{}) tile=({},{})",
        screen_base, hofs, vofs, map_x, map_y, tile_x, tile_y
    );
    println!("get_screen_entry = 0x{:04X}", entry);
}
