use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Run 10 frames and check counts
    for frame in 0..10 {
        let save_before = gba.cpu().irq_save_count;
        let restore_before = gba.cpu().irq_restore_count;
        gba.run_frame_parallel(&mut fb);
        let saves = gba.cpu().irq_save_count - save_before;
        let restores = gba.cpu().irq_restore_count - restore_before;
        let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
        println!(
            "Frame {}: new_saves={} new_restores={} total_save={} total_restore={} nonzero={}",
            frame,
            saves,
            restores,
            gba.cpu().irq_save_count,
            gba.cpu().irq_restore_count,
            nonzero
        );
    }

    // Run to frame 100
    let save_before_100 = gba.cpu().irq_save_count;
    let restore_before_100 = gba.cpu().irq_restore_count;
    for _ in 10..100 {
        gba.run_frame_parallel(&mut fb);
    }
    let save_delta = gba.cpu().irq_save_count - save_before_100;
    let restore_delta = gba.cpu().irq_restore_count - restore_before_100;
    println!(
        "Frame 100: new_saves={} new_restores={} total_save={} total_restore={}",
        save_delta,
        restore_delta,
        gba.cpu().irq_save_count,
        gba.cpu().irq_restore_count
    );
}
