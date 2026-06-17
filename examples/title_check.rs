use rgba::Gba;

fn save_bmp(fb: &[u32], path: &str) {
    let width = 240u32;
    let height = 160u32;
    let row_size = (width * 3 + 3) & !3;
    let pixel_size = row_size * height;
    let file_size = 54 + pixel_size;
    let mut out = Vec::with_capacity(file_size as usize);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&file_size.to_le_bytes());
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&width.to_le_bytes());
    out.extend_from_slice(&height.to_le_bytes());
    out.extend_from_slice(&[1, 0, 24, 0]);
    out.extend_from_slice(&[0; 4]);
    out.extend_from_slice(&pixel_size.to_le_bytes());
    out.extend_from_slice(&[0; 16]);
    out.extend_from_slice(&[0; 16]);
    for y in (0..height).rev() {
        let row_start = (y * width) as usize;
        let mut row = Vec::with_capacity(row_size as usize);
        for x in 0..width {
            let pixel = fb[row_start + x as usize];
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            row.extend_from_slice(&[b, g, r]);
        }
        while row.len() < row_size as usize {
            row.push(0);
        }
        out.extend_from_slice(&row);
    }
    std::fs::write(path, &out).unwrap();
}

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..300 {
        gba.run_frame_parallel(&mut fb);
        if frame == 195 || frame == 250 || frame == 299 {
            let path = format!("/tmp/title_f{}.bmp", frame);
            save_bmp(&fb, &path);
            println!("Saved frame {}", frame);
        }
    }

    for f in ["title_f195", "title_f250", "title_f299"] {
        let bmp = format!("/tmp/{}.bmp", f);
        let png = format!("/tmp/{}.png", f);
        std::process::Command::new("convert")
            .args([&bmp, &png])
            .status()
            .expect("convert failed");
    }

    gba.sync_ppu_full();
    let ppu = gba.ppu();
    let vram = ppu.vram();
    println!("\nFrame 300 state:");
    println!(
        "DISPCNT={:#06X} mode={}",
        ppu.get_dispcnt(),
        ppu.get_display_mode()
    );
    for bg in 0..4 {
        let bgcnt = ppu.get_bgcnt(bg);
        println!(
            "BG{}: cnt={:#06X} tile={:#X} map={:#X} hofs={} vofs={}",
            bg,
            bgcnt,
            ppu.get_bg_tile_base(bg),
            ppu.get_bg_map_base(bg),
            ppu.get_bg_hofs(bg),
            ppu.get_bg_vofs(bg)
        );
    }

    let mut nonzero = 0;
    for tile in 0..512 {
        let off = tile * 32;
        let mut has_data = false;
        for b in 0..32 {
            if vram[off + b] != 0 && vram[off + b] != 0xAA {
                has_data = true;
                break;
            }
        }
        if has_data {
            nonzero += 1;
        }
    }
    println!(
        "\n{} tiles with non-zero non-0xAA data in char_block 0",
        nonzero
    );
}
