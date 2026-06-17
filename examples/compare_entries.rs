use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }

    let bg0cnt = gba.mem.read_half(0x04000008);
    let screen_base_block = ((bg0cnt >> 8) & 0x1F) as u32;
    let screen_base_addr = 0x06000000 + screen_base_block * 0x800;

    println!("BG0CNT: 0x{:04X}", bg0cnt);
    println!("Screen base block: {}", screen_base_block);
    println!("Screen base addr: 0x{:08X}", screen_base_addr);
    println!();
    println!("First 64 screen entries:");

    for i in 0..64 {
        let addr = screen_base_addr + (i as u32) * 2;
        let entry = gba.mem.read_half(addr);
        println!(
            "  [{:2}] @ 0x{:08X} = 0x{:04X} (tile={}, pal={}, hflip={}, vflip={})",
            i,
            addr,
            entry,
            entry & 0x3FF,
            (entry >> 12) & 0xF,
            (entry >> 10) & 1,
            (entry >> 11) & 1,
        );
    }
}
