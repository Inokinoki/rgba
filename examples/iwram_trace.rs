use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    println!("IWRAM[0x00-0x0F] evolution:");
    for frame in 0..8 {
        gba.run_frame_parallel(&mut fb);
        let iwram = gba.mem().iwram();
        let w0 = u32::from_le_bytes([iwram[0], iwram[1], iwram[2], iwram[3]]);
        let w4 = u32::from_le_bytes([iwram[4], iwram[5], iwram[6], iwram[7]]);
        let b8 = iwram[8];
        let w12 = u32::from_le_bytes([iwram[0x0C], iwram[0x0D], iwram[0x0E], iwram[0x0F]]);
        let pc = gba.cpu().get_pc();
        println!(
            "F{}: [0000]=0x{:08X} [0004]=0x{:08X} [0008]=0x{:02X} [000C]=0x{:08X} PC=0x{:08X}",
            frame, w0, w4, b8, w12, pc
        );
    }
}
