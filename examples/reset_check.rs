use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    let mut reset_count = 0;

    for frame in 0..400 {
        let pc_before = gba.cpu().get_pc();
        gba.run_frame_parallel(&mut fb);
        let pc_after = gba.cpu().get_pc();

        if pc_after == 0x08000000 && pc_before != 0x08000000 && frame > 0 {
            reset_count += 1;
            println!(
                "Frame {}: RESET detected! PC went from {:#010X} to {:#010X}",
                frame, pc_before, pc_after
            );
        }
    }
    println!("\nTotal resets: {}", reset_count);
}
