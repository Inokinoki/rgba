use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..5 {
        gba.run_frame_parallel(&mut fb);
        let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
        let cpu = gba.cpu();
        eprintln!(
            "Frame {}: {} pix, mode={:?} PC={:08X} SP={:08X} thumb={}",
            frame,
            nonzero,
            cpu.get_mode(),
            cpu.get_pc(),
            cpu.get_sp(),
            cpu.is_thumb_mode()
        );
    }
}
