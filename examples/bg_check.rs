use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }

    for bg_idx in 0..4 {
        let bgcnt = gba.mem.read_half(0x04000008 + bg_idx * 2);
        let pri = bgcnt & 3;
        let cb = ((bgcnt >> 2) & 3) as usize * 0x4000;
        let tb = ((bgcnt >> 8) & 0x1F) as usize * 0x800;
        let hscroll = gba.mem.read_half(0x04000010 + bg_idx * 4);
        let vscroll = gba.mem.read_half(0x04000012 + bg_idx * 4);
        let vram = gba.mem.vram();
        let scroll_x = (hscroll & 0x1FF) as usize;
        let tile_col = scroll_x / 8;
        let mut entries = Vec::new();
        for row in 0..2 {
            for col in tile_col..tile_col + 4 {
                let entry_off = tb + (row * 64 + col) * 2;
                if entry_off + 1 < 0x18000 {
                    let entry = u16::from_le_bytes([vram[entry_off], vram[entry_off + 1]]);
                    entries.push(format!("{:04X}", entry));
                }
            }
        }
        println!(
            "BG{}: pri={} cb=0x{:04X} tb=0x{:04X} scroll=({},{}) visible_tiles={:?}",
            bg_idx,
            pri,
            cb,
            tb,
            hscroll,
            vscroll,
            &entries[..std::cmp::min(entries.len(), 8)]
        );
    }

    println!("\nPalette banks:");
    let palette = gba.mem.palette();
    for bank in 0..4 {
        let off = bank * 32;
        let c0 = u16::from_le_bytes([palette[off], palette[off + 1]]);
        let c1 = u16::from_le_bytes([palette[off + 2], palette[off + 3]]);
        println!("  Bank {}: {:04X} {:04X} ...", bank, c0, c1);
    }
}
