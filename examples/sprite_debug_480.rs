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
    let mut enabled = 0usize;
    let mut window = 0usize;
    for s in 0..128usize {
        let is_en = ppu.sprite_is_enabled(s);
        let is_win = ppu.sprite_is_window(s);
        let x = ppu.sprite_x(s);
        let y = ppu.sprite_y(s);
        if is_en {
            enabled += 1;
            if is_win {
                window += 1;
            }
            let (w, h) = ppu.sprite_dimensions(s);
            let prio = ppu.sprite_priority(s);
            let tile = ppu.sprite_tile(s);
            let affine = ppu.sprite_is_affine(s);
            let pal = if !affine { ppu.sprite_palette(s) } else { 0 };
            println!(
                "S{:3}: win={} x={:3} y={:3} {}x{} tile={:4} pal={} pri={} aff={}",
                s, is_win, x, y, w, h, tile, pal, prio, affine
            );
        }
    }
    println!("enabled={} window={}", enabled, window);
}
