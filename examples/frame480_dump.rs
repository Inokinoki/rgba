use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..480 {
        gba.run_frame_parallel(&mut fb);
    }

    let ppu = &gba.ppu;
    for s in 0..128 {
        if !ppu.sprite_is_enabled(s) || ppu.sprite_is_window(s) {
            continue;
        }
        let (w, h) = ppu.sprite_dimensions(s);
        let x = ppu.sprite_x(s);
        let y = ppu.sprite_y(s);
        let prio = ppu.sprite_priority(s);
        let tile = ppu.sprite_tile(s);
        let is_affine = ppu.sprite_is_affine(s);
        let palette = if !is_affine { ppu.sprite_palette(s) } else { 0 };
        let dbl = if is_affine {
            ppu.sprite_double_size(s)
        } else {
            false
        };
        println!(
            "Sprite {:3}: x={:3} y={:3} w={:3} h={:3} tile={:4} pal={} prio={} affine={} dbl={}",
            s, x, y, w, h, tile, palette, prio, is_affine, dbl
        );
    }

    let mut ppm = String::from("P3\n240 160\n255\n");
    for y in 0..160 {
        for x in 0..240 {
            let c = fb[y * 240 + x];
            let r = (c >> 16) & 0xFF;
            let g = (c >> 8) & 0xFF;
            let b = c & 0xFF;
            ppm.push_str(&format!("{} {} {} ", r, g, b));
        }
        ppm.push('\n');
    }
    std::fs::write("/tmp/frame480_with_sprites.ppm", ppm).unwrap();
    println!("Saved /tmp/frame480_with_sprites.ppm");
}
