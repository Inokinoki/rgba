use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Check IRQ handler pointer at various frames
    for frame in 0..400 {
        gba.run_frame_parallel(&mut fb);
        if frame % 50 == 0 || frame == 1 {
            let handler_ptr = gba.mem.read_word(0x03007FFC);
            let vbl_cnt = gba.mem.read_word(0x03007FF8);
            println!("Frame {:4}: IRQ handler ptr = {:08X} VBlankCnt = {:08X}", frame, handler_ptr, vbl_cnt);
        }
    }
}
