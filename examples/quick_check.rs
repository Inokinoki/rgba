use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..10 {
        gba.run_frame_parallel(&mut fb);
        let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
        eprintln!(
            "Frame {}: {} pix, saves={}, restores={}",
            frame,
            nonzero,
            gba.cpu().irq_save_count,
            gba.cpu().irq_restore_count
        );
    }
}
