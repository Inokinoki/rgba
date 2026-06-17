use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for frame in 0..200 {
        for _scanline in 0..228 {
            gba.run_scanline();
        }
    }

    // Check timer 0 value - it's running but with IRQ disabled
    let io = gba.mem().io();
    let tm0cnt_l = u16::from_le_bytes([io[0x100], io[0x101]]);
    let tm0cnt_h = io[0x102];
    let tm1cnt_l = u16::from_le_bytes([io[0x104], io[0x105]]);
    let tm1cnt_h = io[0x106];

    println!("TM0: count={:04X} ctrl={:02X}", tm0cnt_l, tm0cnt_h);
    println!("TM1: count={:04X} ctrl={:02X}", tm1cnt_l, tm1cnt_h);

    // Let me check what value the game's VBlank handler reads from
    // EWRAM to determine if it should load more data
    // The VBlank handler at frame 191+ reads from 0x0200918C (r0)
    let wram = gba.mem().wram();
    let off = 0x918C;
    if off + 32 <= wram.len() {
        println!("\nEWRAM at 0x0200918C: {:02X?}", &wram[off..off + 32]);
    }

    // Check 0x020089C4 area (referenced at frame 190 SL 177)
    let off2 = 0x89C4;
    if off2 + 32 <= wram.len() {
        println!("EWRAM at 0x020089C4: {:02X?}", &wram[off2..off2 + 32]);
    }

    // Check 0x02008D2C area
    let off3 = 0x8D2C;
    if off3 + 32 <= wram.len() {
        println!("EWRAM at 0x02008D2C: {:02X?}", &wram[off3..off3 + 32]);
    }

    // Check IWRAM at 0x03007D24 (referenced at frame 191 SL 176)
    let iwram = gba.mem().iwram();
    println!("\nIWRAM at 0x03007D24: {:02X?}", &iwram[0x7D24..0x7D44]);

    // Let me also check what the decompression table looks like
    // The function at 0x080D1AA0 uses ldr r2, [pc, #336] and ldr r1, [pc, #32]
    // to load table base addresses. Let me find those addresses.
    // At offset 0x080D1AA0+0x34 = 0x080D1AD4: literal pool
    let rom = gba.mem().rom();
    let table_off = 0x000D1AD4;
    if table_off + 8 <= rom.len() {
        let v1 = u32::from_le_bytes([
            rom[table_off],
            rom[table_off + 1],
            rom[table_off + 2],
            rom[table_off + 3],
        ]);
        let v2 = u32::from_le_bytes([
            rom[table_off + 4],
            rom[table_off + 5],
            rom[table_off + 6],
            rom[table_off + 7],
        ]);
        println!("\nTable base addresses: {:08X} {:08X}", v1, v2);
    }
}
