use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..400 {
        gba.run_frame_parallel(&mut fb);
    }

    println!("ARM SWI calls: {}", gba.mem().arm_swi_count);
    println!("THUMB SWI calls: {}", gba.mem().thumb_swi_count);
}
