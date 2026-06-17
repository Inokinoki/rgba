use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..192u32 {
        gba.run_frame_parallel(&mut fb);
    }

    let ewram = gba.mem.wram();

    // Check same ranges as mGBA overview
    println!("=== Our EWRAM overview ===");
    for base in (0x8000..0xA000).step_by(0x100) {
        let off = base;
        if off + 4 <= ewram.len() {
            let val =
                u32::from_le_bytes([ewram[off], ewram[off + 1], ewram[off + 2], ewram[off + 3]]);
            if val != 0 {
                println!("  {:08X}: {:08X} *", 0x02000000 + base, val);
            }
        }
    }

    // Check specific OBJ palette source region
    println!("\n=== Our EWRAM at 0x0200883C-0x0200891C (where mGBA has palette data) ===");
    let start = 0x883C;
    let mut nonzero = 0;
    for i in 0..57 {
        let off = start + i * 4;
        if off + 4 <= ewram.len() {
            let val =
                u32::from_le_bytes([ewram[off], ewram[off + 1], ewram[off + 2], ewram[off + 3]]);
            if val != 0 {
                nonzero += 1;
            }
        }
    }
    println!("Non-zero words in mGBA's data range: {}/57", nonzero);

    // Count total non-zero words in EWRAM 0x8000-0xA000
    let mut total_nz = 0;
    for i in (0x8000..0xA000).step_by(4) {
        if i + 4 <= ewram.len() {
            let val = u32::from_le_bytes([ewram[i], ewram[i + 1], ewram[i + 2], ewram[i + 3]]);
            if val != 0 {
                total_nz += 1;
            }
        }
    }
    println!(
        "\nTotal non-zero words in EWRAM 0x2008000-0x200A000: {}",
        total_nz
    );
}
