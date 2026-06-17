use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..460 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = &gba.ppu;
    let dispcnt = ppu.get_dispcnt();
    let obj_enabled = (dispcnt & (1 << 12)) != 0;
    println!("OBJ enabled: {}", obj_enabled);

    let mut sprite_count = 0;
    let mut text_like_sprites = 0;
    for s in 0..128 {
        if !ppu.sprite_is_enabled(s) || ppu.sprite_is_window(s) {
            continue;
        }
        sprite_count += 1;
        let (w, h) = ppu.sprite_dimensions(s);
        let x = ppu.sprite_x(s);
        let y = ppu.sprite_y(s);
        let prio = ppu.sprite_priority(s);
        let is_affine = ppu.sprite_is_affine(s);
        let tile = ppu.sprite_tile(s);
        let palette = if !is_affine { ppu.sprite_palette(s) } else { 0 };

        if y < 60 && x < 200 {
            text_like_sprites += 1;
            println!(
                "Sprite {:3}: x={:3} y={:3} w={:3} h={:3} tile={:4} pal={} prio={} affine={}",
                s, x, y, w, h, tile, palette, prio, is_affine
            );
        }
    }
    println!(
        "Total sprites: {}, text-like (y<60, x<200): {}",
        sprite_count, text_like_sprites
    );

    let mut obj_only = String::from("P3\n240 160\n255\n");
    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = if let Some((color, _)) = gba.get_sprite_pixel(ppu, x, y) {
                let r = ((color >> 10) & 0x1F) as u32 * 255 / 31;
                let g = ((color >> 5) & 0x1F) as u32 * 255 / 31;
                let b = (color & 0x1F) as u32 * 255 / 31;
                format!("{} {} {} ", r, g, b)
            } else {
                "32 32 32 ".to_string()
            };
            obj_only.push_str(&c);
        }
        obj_only.push('\n');
    }
    std::fs::write("/tmp/obj_only_460.ppm", obj_only).unwrap();
    println!("Saved OBJ-only to /tmp/obj_only_460.ppm");
}
