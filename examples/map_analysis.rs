use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let vram = gba.mem().vram();

    println!("=== BG3 map at 0xF800 (32x32 entries, 1024 entries) ===");
    println!("Screen base 0xF800 = VRAM offset 0xF800");
    let map_base = 0xF800;

    let mut tile_ref_counts: [u32; 1024] = [0; 1024];
    for i in 0..1024usize {
        let off = map_base + i * 2;
        if off + 2 <= vram.len() {
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile = (entry & 0x3FF) as usize;
            tile_ref_counts[tile] += 1;
        }
    }

    let mut tiles_used = 0;
    let mut max_tile = 0;
    for t in 0..1024 {
        if tile_ref_counts[t] > 0 {
            tiles_used += 1;
            max_tile = t;
        }
    }
    println!("Unique tiles referenced: {}", tiles_used);
    println!("Max tile: {}", max_tile);

    println!("\n=== Palette analysis ===");
    let pal = gba.mem().palette();
    println!("First 16 palette entries:");
    for i in 0..16usize {
        let color = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        let r = color & 0x1F;
        let g = (color >> 5) & 0x1F;
        let b = (color >> 10) & 0x1F;
        print!("  [{:2}]={:04X}({},{},{})", i, color, r, g, b);
        if i % 4 == 3 {
            println!();
        }
    }

    println!("\n=== First 10x10 of BG3 map ===");
    for y in 0..10u32 {
        for x in 0..10u32 {
            let i = y * 32 + x;
            let off = map_base + i as usize * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            print!("{:3}", tile);
        }
        println!();
    }

    println!("\n=== Test: Check if BG3 is the title screen BG ===");
    println!("BG3 at frame 195 should be the title/cutscene background");

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    let bg3cnt = u16::from_le_bytes([io[0x0E], io[0x0F]]);
    let bg3_enabled = (dispcnt >> 11) & 1;
    let bg3_priority = bg3cnt & 3;
    let bg3_char_base = ((bg3cnt >> 2) & 3) as u32 * 0x4000;
    let bg3_screen_base = ((bg3cnt >> 8) & 0x1F) as u32 * 0x800;
    let bg3_size = (bg3cnt >> 14) & 3;
    let bg3_mosaic = (bg3cnt >> 6) & 1;
    let bg3_256color = (bg3cnt >> 7) & 1;
    println!(
        "BG3: enabled={} priority={} char={:#X} screen={:#X} size={} mosaic={} {}bit",
        bg3_enabled,
        bg3_priority,
        bg3_char_base,
        bg3_screen_base,
        bg3_size,
        bg3_mosaic,
        if bg3_256color != 0 { "256" } else { "16" }
    );

    let scx = u16::from_le_bytes([io[0x18], io[0x19]]);
    let scy = u16::from_le_bytes([io[0x1A], io[0x1B]]);
    println!("BG3 scroll: X={} Y={}", scx & 0x1FF, scy & 0x1FF);
}
