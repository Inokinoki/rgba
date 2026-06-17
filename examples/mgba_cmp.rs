use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..351 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("=== Our state at frame 350 ===");
    let io = gba.mem.io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    let dispstat = u16::from_le_bytes([io[4], io[5]]);
    let vcount = u16::from_le_bytes([io[6], io[7]]);
    println!("DISPCNT={:04X} DISPSTAT={:04X} VCOUNT={:04X}", dispcnt, dispstat, vcount);
    println!("IE={:04X} IF={:04X} IME={}", gba.mem.interrupt.ie.bits(), gba.mem.interrupt.if_raw.bits(), gba.mem.interrupt.ime);
    println!("KEYINPUT={:04X} KEYCNT={:04X}", u16::from_le_bytes([io[0x130], io[0x131]]), u16::from_le_bytes([io[0x132], io[0x133]]));
    
    for t in 0..4 {
        let base = 0x100 + t * 4;
        let cnt = u16::from_le_bytes([io[base], io[base + 1]]);
        let ctrl = u16::from_le_bytes([io[base + 2], io[base + 3]]);
        println!("Timer{}: {:04X} {:04X}", t, cnt, ctrl);
    }

    println!("\nKey memory:");
    for addr in [0x02000050, 0x02000068, 0x02000074, 0x02000090, 0x020000A0, 0x020000F0, 0x03007FF8] {
        println!("  {:08X}: {:08X}", addr, gba.mem.read_word(addr));
    }
}
