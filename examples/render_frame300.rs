use rgba::Gba;
use std::io::Write;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..300 {
        for _ in 0..228 {
            gba.run_scanline();
        }
    }

    gba.sync_ppu_full();
    for y in 0..160u16 {
        for x in 0..240u16 {
            let color = gba.get_pixel_tile_mode(x, y);
            let r = ((color & 0x1F) as u32 * 255 / 31) << 16;
            let g = (((color >> 5) & 0x1F) as u32 * 255 / 31) << 8;
            let b = ((color >> 10) & 0x1F) as u32 * 255 / 31;
            fb[(y as usize) * 240 + (x as usize)] = r | g | b;
        }
    }

    let w = 240u32;
    let h = 160u32;
    let row_size = (w * 4 + 3) & !3;
    let img_size = row_size * h;
    let file_size = 54 + img_size;

    let mut bmp = Vec::new();
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&54u32.to_le_bytes());
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&w.to_le_bytes());
    bmp.extend_from_slice(&h.to_le_bytes());
    bmp.extend_from_slice(&[1, 0, 32, 0]);
    bmp.extend_from_slice(&[0; 4]);
    bmp.extend_from_slice(&img_size.to_le_bytes());
    bmp.extend_from_slice(&[0x13, 0x0B, 0, 0, 0x13, 0x0B, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    for y in (0..h).rev() {
        for x in 0..w {
            let pixel = fb[(y * w + x) as usize];
            bmp.extend_from_slice(&pixel.to_le_bytes());
        }
        let padding = row_size - w * 4;
        bmp.extend_from_slice(&vec![0; padding as usize]);
    }

    let mut f = std::fs::File::create("/tmp/frame300.bmp").unwrap();
    f.write_all(&bmp).unwrap();
    println!("Written /tmp/frame300.bmp");
}
