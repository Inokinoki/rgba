use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];
    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let bldcnt = gba.ppu.get_blend_control();
    let bldalpha = gba.ppu.get_blend_alpha();
    let bldy = gba.ppu.get_blend_brightness();
    let blend_mode = gba.ppu.get_blend_mode();

    println!("BLDCNT: 0x{:04X} (mode={})", bldcnt, blend_mode);
    println!("BLDALPHA: 0x{:04X}", bldalpha);
    println!("BLDY: 0x{:04X} (ey={})", bldy, bldy & 0x1F);

    let io = gba.mem.io();
    let bldy_io = u16::from_le_bytes([io[0x54], io[0x55]]);
    println!("IO[0x54]: 0x{:04X}", bldy_io);

    // The brightness increase formula:
    // new = c + (31 - c) * ey / 16
    // With 0x7E80 and ey=?
    let c = 0x7E80u16;
    let r = c & 0x1F; // 0
    let g = (c >> 5) & 0x1F; // 20
    let b = (c >> 10) & 0x1F; // 15

    // If ey = 16 (max): r=31, g=31, b=31 = white
    // If ey = 8: r=0+(31-0)*8/16=15, g=20+(31-20)*8/16=25, b=15+(31-15)*8/16=23
    // Result = 15 | (25<<5) | (23<<10) = 0x7CCF

    // Actual result is 0x7F99 = 0111_1111_1001_1001
    // r = 0x1F = 31
    // g = 0x1C = 28
    // b = 0x1E = 30
    // This means ey is large (close to 16)

    // With ey=16: r=31, g=20+(11)*16/16=31, b=15+(16)*16/16=31 -> 0x7FFF
    // With ey=12: r=0+(31)*12/16=23, g=20+(11)*12/16=28, b=15+(16)*12/16=27
    // = 23 | (28<<5) | (27<<10) = 23 | 896 | 27648 = 0x6DB7
    // Hmm that doesn't match either

    // 0x7F99: r=25, g=28, b=30
    // From r: 0 + 31*ey/16 = 25 -> ey = 25*16/31 = 12.9 -> ey=13
    // Let me check with ey=13: r=25, g=20+(11*13/16)=28, b=15+(16*13/16)=28
    // = 25 | (28<<5) | (28<<10) = 0x7E19
    // Still doesn't match... let me just check the IO register

    // Actually let's compute what ey makes 0x7E80 -> 0x7F99
    // 0x7F99: r=25, g=28, b=30
    // From r=0: 0 + 31*ey/16 = 25 -> ey = 25*16/31 ≈ 12.9
    // From g=20: 20 + 11*ey/16 = 28 -> ey = 8*16/11 ≈ 11.6
    // From b=15: 15 + 16*ey/16 = 30 -> ey = 15
    // These are inconsistent... the formula might be different

    // Actually BLDY is at IO 0x04000054, which is write-only
    // Let me check if our write_io handler stores BLDY correctly
    println!("\nChecking IO BLDY storage...");
    println!("IO[0x54]=0x{:02X} IO[0x55]=0x{:02X}", io[0x54], io[0x55]);
}
