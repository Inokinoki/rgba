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

    // Read the PPU snapshot and test rendering manually
    // Check what PPU snapshot looks like
    let dispcnt = gba.mem.read_half(0x04000000);
    let bg0cnt = gba.mem.read_half(0x04000008);
    let bg0hofs = gba.mem.read_half(0x04000010) & 0x1FF;
    let bg0vofs = gba.mem.read_half(0x04000012) & 0x1FF;

    println!("Rendering test at screen (3,3):");
    println!(
        "  BG0CNT=0x{:04X} hofs={} vofs={}",
        bg0cnt, bg0hofs, bg0vofs
    );

    // Calculate map coordinates
    let map_w = 512u16; // size=1 => 64x32 tiles = 512 wide
    let px = (3u16.wrapping_add(bg0hofs)) % map_w;
    let py = (3u16.wrapping_add(bg0vofs)) % 256;
    let tile_x = px / 8;
    let tile_y = py / 8;
    let pix_x = (px % 8) as usize;
    let pix_y = (py % 8) as usize;

    println!(
        "  map=({},{}) tile=({},{}) pixel=({},{})",
        px, py, tile_x, tile_y, pix_x, pix_y
    );

    // Look up tile entry (size=1 => 64x32 => 2 screen blocks horizontally)
    let screen_base = 0xC000usize;
    let block_x = (tile_x / 32) as usize;
    let block_num = block_x; // only 1 row of blocks for 64x32
    let local_x = (tile_x % 32) as usize;
    let local_y = (tile_y % 32) as usize;
    let entry_off = screen_base + block_num * 0x800 + (local_y * 32 + local_x) * 2;

    let vram = gba.mem.vram();
    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
    let tile_num = entry & 0x3FF;
    let pal = (entry >> 12) & 0xF;
    println!("  entry=0x{:04X} tile={} pal={}", entry, tile_num, pal);

    // Get tile pixel
    let tile_offset = (tile_num as usize) * 32;
    let row_offset = tile_offset + pix_y * 4;
    let byte_off = row_offset + pix_x / 2;
    let byte = vram[byte_off];
    let color_idx = if pix_x % 2 == 0 {
        byte & 0xF
    } else {
        (byte >> 4) & 0xF
    };
    println!(
        "  byte_off={} byte=0x{:02X} color_idx={}",
        byte_off, byte, color_idx
    );

    // Palette lookup
    let palette = gba.mem.palette();
    let pal_off = pal as usize * 32 + color_idx as usize * 2;
    let color = u16::from_le_bytes([palette[pal_off], palette[pal_off + 1]]);
    println!("  palette[{}][{}] = 0x{:04X}", pal, color_idx, color);

    // Convert to RGB
    let r = (color & 0x1F) as u32 * 255 / 31;
    let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
    let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
    println!("  RGB=({},{},{}) = 0x{:02X}{:02X}{:02X}", r, g, b, r, g, b);

    // Compare with actual framebuffer
    let fb_color = fb[3 * 240 + 3] & 0xFFFFFF;
    println!("\n  Actual FB at (3,3): 0x{:06X}", fb_color);

    // Check what the scanline renderer actually produces
    // The key is: does sync_ppu_to_mem correctly copy the IO registers?
    // And does the PPU snapshot contain the correct data at render time?
    println!("\n=== Checking sync timing ===");
    // The run_frame_parallel calls run_scanline for 228 scanlines
    // sync_ppu_to_mem is called at the end of each scanline
    // But the PPU renders using a snapshot taken at the START of the frame

    // Let's check what the actual snapshot looks like
    // by reading the final framebuffer for the full screen
    let mut screen_colors: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for &c in fb.iter() {
        *screen_colors.entry(c & 0xFFFFFF).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = screen_colors.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    println!("Top 10 FB colors:");
    for (i, (color, count)) in sorted.iter().take(10).enumerate() {
        println!("  {:2}: 0x{:06X} ({} pixels)", i + 1, color, count);
    }
}
