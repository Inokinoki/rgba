use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..20 {
        gba.run_frame_parallel(&mut fb);
        let iwram = gba.mem().iwram();
        let count = u32::from_le_bytes([iwram[0x7FF8], iwram[0x7FF9], iwram[0x7FFA], iwram[0x7FFB]]);
        let dispcnt = gba.ppu().get_dispcnt();
        println!("Frame {:3}: counter={:#010X} DISPCNT={:04X}", frame + 1, count, dispcnt);
    }
}
