use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.sync_ppu_full();

    let ppu = gba.ppu();
    for i in 0..20 {
        let enabled = ppu.sprite_is_enabled(i);
        let y = ppu.sprite_y(i);
        let x = ppu.sprite_x(i);
        let (w, h) = ppu.sprite_dimensions(i);
        let tile = ppu.sprite_tile(i);
        let prio = ppu.sprite_priority(i);
        let is_256 = ppu.sprite_is_256color(i);
        let pal = ppu.sprite_palette(i);
        let affine = ppu.sprite_is_affine(i);

        if enabled && y < 160 {
            println!(
                "Sprite {:2}: y={:3} x={:3} w={} h={} tile={} pri={} 256c={} pal={} aff={}",
                i, y, x, w, h, tile, prio, is_256, pal, affine
            );
        }
    }

    println!("\nSprite pixel tests:");
    for (x, y) in [(100u16, 116u16), (104u16, 145u16), (20u16, 70u16)] {
        match gba.get_sprite_pixel(ppu, x, y) {
            Some((color, prio)) => {
                println!("  ({}, {}) = color={:04X} prio={}", x, y, color, prio);
            }
            None => {
                println!("  ({}, {}) = None", x, y);
            }
        }
    }
}
