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

    // Read PPU registers
    let dispcnt = gba.mem.read_half(0x04000000);
    let bg0cnt = gba.mem.read_half(0x04000008);
    let bg1cnt = gba.mem.read_half(0x0400000A);
    let bg2cnt = gba.mem.read_half(0x0400000C);
    let bg3cnt = gba.mem.read_half(0x0400000E);

    println!("Frame 300 PPU state:");
    println!(
        "DISPCNT: {:04X} (mode={}, bg0={}, bg1={}, bg2={}, bg3={}, obj={})",
        dispcnt,
        dispcnt & 7,
        (dispcnt >> 8) & 1,
        (dispcnt >> 9) & 1,
        (dispcnt >> 10) & 1,
        (dispcnt >> 11) & 1,
        (dispcnt >> 12) & 1
    );
    println!(
        "BG0CNT: {:04X} (pri={}, cb=0x{:X}, tb=0x{:X}, size={})",
        bg0cnt,
        bg0cnt & 3,
        (bg0cnt >> 2) & 3,
        (bg0cnt >> 8) & 0x1F,
        (bg0cnt >> 14) & 3
    );
    println!(
        "BG1CNT: {:04X} (pri={}, cb=0x{:X}, tb=0x{:X}, size={})",
        bg1cnt,
        bg1cnt & 3,
        (bg1cnt >> 2) & 3,
        (bg1cnt >> 8) & 0x1F,
        (bg1cnt >> 14) & 3
    );
    println!(
        "BG2CNT: {:04X} (pri={}, cb=0x{:X}, tb=0x{:X}, size={})",
        bg2cnt,
        bg2cnt & 3,
        (bg2cnt >> 2) & 3,
        (bg2cnt >> 8) & 0x1F,
        (bg2cnt >> 14) & 3
    );
    println!(
        "BG3CNT: {:04X} (pri={}, cb=0x{:X}, tb=0x{:X}, size={})",
        bg3cnt,
        bg3cnt & 3,
        (bg3cnt >> 2) & 3,
        (bg3cnt >> 8) & 0x1F,
        (bg3cnt >> 14) & 3
    );

    // Check BG scrolling
    let bg0hx = gba.mem.read_half(0x04000010);
    let bg0hy = gba.mem.read_half(0x04000012);
    let bg1hx = gba.mem.read_half(0x04000014);
    let bg1hy = gba.mem.read_half(0x04000016);
    println!(
        "BG0 hscroll: {:04X},{:04X}  BG1 hscroll: {:04X},{:04X}",
        bg0hx, bg0hy, bg1hx, bg1hy
    );

    // Check tile data in VRAM at the text BG's tile base
    let bg0_tile_base = ((bg0cnt >> 8) & 0x1F) as usize * 0x800;
    let bg0_char_base = ((bg0cnt >> 2) & 3) as usize * 0x4000;
    println!(
        "\nBG0 tile base: 0x{:X}  char base: 0x{:X}",
        bg0_tile_base, bg0_char_base
    );

    // Check if there are non-zero tiles in the character base
    let mut nonzero = 0;
    for i in 0..0x1000 {
        if gba.mem.vram()[bg0_char_base + i] != 0 {
            nonzero += 1;
        }
    }
    println!("Non-zero bytes in BG0 char area: {}/4096", nonzero);

    // Check tile map entries
    let mut nonzero_tiles = 0;
    for i in (0..0x400).step_by(2) {
        let entry = u16::from_le_bytes([
            gba.mem.vram()[bg0_tile_base + i],
            gba.mem.vram()[bg0_tile_base + i + 1],
        ]);
        if entry != 0 {
            nonzero_tiles += 1;
            if nonzero_tiles <= 10 {
                let tile_idx = entry & 0x3FF;
                let hflip = (entry >> 10) & 1;
                let vflip = (entry >> 11) & 1;
                let palette = (entry >> 12) & 0xF;
                println!(
                    "  Tile entry {:04X} (tile={}, hf={}, vf={}, pal={})",
                    entry, tile_idx, hflip, vflip, palette
                );
            }
        }
    }
    println!("Non-zero tile map entries in BG0: {}/512", nonzero_tiles);
}
