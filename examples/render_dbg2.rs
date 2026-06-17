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

    let dispcnt = gba.mem.read_half(0x04000000);
    println!(
        "DISPCNT: 0x{:04X} mode={} bg0={} bg1={} bg2={} bg3={} obj={}",
        dispcnt,
        dispcnt & 7,
        (dispcnt >> 8) & 1,
        (dispcnt >> 9) & 1,
        (dispcnt >> 10) & 1,
        (dispcnt >> 11) & 1,
        (dispcnt >> 12) & 1
    );

    for y in [0u32, 4, 8, 16, 32, 48, 64] {
        let mut colors = Vec::new();
        for x in [0, 8, 32, 64, 120, 160, 200, 239] {
            let c = fb[(y * 240 + x) as usize] & 0xFFFFFF;
            colors.push(format!("0x{:06X}", c));
        }
        println!("Row {}: {}", y, colors.join(" "));
    }

    let bg0h = gba.mem.read_half(0x04000010) & 0x1FF;
    let bg0v = gba.mem.read_half(0x04000012) & 0x1FF;
    let bg3h = gba.mem.read_half(0x0400001C) & 0x1FF;
    let bg3v = gba.mem.read_half(0x0400001E) & 0x1FF;
    println!("\nBG0 scroll: h={} v={}", bg0h, bg0v);
    println!("BG3 scroll: h={} v={}", bg3h, bg3v);

    let palette = gba.mem.palette();
    let backdrop = u16::from_le_bytes([palette[0], palette[1]]);
    println!("Backdrop: 0x{:04X}", backdrop);
    println!("FB at (0,0): 0x{:06X}", fb[0] & 0xFFFFFF);
}
