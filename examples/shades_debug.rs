use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/ppu/shades.gba")
        .unwrap();

    for _ in 0..500 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let dc = gba.ppu().get_dispcnt();
    println!("DISPCNT: {:#06X} mode={}", dc, dc & 0x7);

    for bg in 0..4 {
        let enabled = (dc >> (8 + bg)) & 1;
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let pri = bgcnt & 3;
        let tile_base = ((bgcnt >> 2) & 3) * 0x4000;
        let scr_base = ((bgcnt >> 8) & 0x1F) * 0x800;
        let size = (bgcnt >> 14) & 3;
        println!(
            "BG{}: en={} pri={} tile_base={:#06X} scr_base={:#06X} size={}",
            bg, enabled, pri, tile_base, scr_base, size
        );
    }

    let vram = gba.ppu().vram();
    let bg0cnt = gba.ppu().get_bgcnt(0);
    let tile_base = ((bg0cnt >> 2) & 3) * 0x4000;
    let scr_base = ((bg0cnt >> 8) & 0x1F) * 0x800;
    let hofs = gba.ppu().get_bg_hofs(0);
    let vofs = gba.ppu().get_bg_vofs(0);
    println!("BG0 hofs={} vofs={}", hofs, vofs);

    println!("\nScreen entries (first 4 rows):");
    for row in 0..4 {
        for col in 0..30 {
            let offset = scr_base as usize + (row * 32 + col) * 2;
            let entry = u16::from_le_bytes([vram[offset], vram[offset + 1]]);
            let tile = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            print!("({},{}) ", tile, pal);
        }
        println!();
    }

    println!("\nTile 0 data at tile_base={:#06X}:", tile_base);
    for row in 0..8 {
        let off = tile_base as usize + row * 4;
        print!("  Row {}: ", row);
        for b in 0..4 {
            print!("{:02X} ", vram[off + b]);
        }
        println!();
    }

    println!("\nTile 1 data:");
    for row in 0..8 {
        let off = tile_base as usize + 32 + row * 4;
        print!("  Row {}: ", row);
        for b in 0..4 {
            print!("{:02X} ", vram[off + b]);
        }
        println!();
    }

    println!("\nPalette (first 32 colors):");
    let pal = gba.mem().palette();
    for i in 0..32 {
        let color = u16::from_le_bytes([pal[i * 2], pal[i * 2 + 1]]);
        let r = (color & 0x1F) as u32 * 255 / 31;
        let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
        let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
        print!("{}:({},{},{}) ", i, r, g, b);
        if i % 8 == 7 {
            println!();
        }
    }

    println!("\nSample pixels (first row of screen):");
    for x in (0..240).step_by(8) {
        let c = gba.get_pixel_tile_mode(x, 0);
        let r = (c & 0x1F) as u32 * 255 / 31;
        let g = ((c >> 5) & 0x1F) as u32 * 255 / 31;
        let b = ((c >> 10) & 0x1F) as u32 * 255 / 31;
        print!("x{}:({},{},{}) ", x, r, g, b);
    }
    println!();
}
