use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut before = vec![0u8; 16];
    for i in 0..16 {
        before[i] = gba.mem_mut().read_byte(0x03000000 + i as u32);
    }
    print!("IWRAM at 0x03000000 BEFORE any frames: ");
    for b in &before {
        print!("{:02X}", b);
    }
    println!();

    let mut fb = vec![0u32; 240 * 160];
    gba.run_frame_parallel(&mut fb);

    let mut after_1 = vec![0u8; 16];
    for i in 0..16 {
        after_1[i] = gba.mem_mut().read_byte(0x03000000 + i as u32);
    }
    print!("IWRAM at 0x03000000 AFTER frame 0:     ");
    for b in &after_1 {
        print!("{:02X}", b);
    }
    println!();

    let rom = gba.mem().rom();
    print!("ROM at 0x080D0DE0:                      ");
    for i in 0..16 {
        print!("{:02X}", rom[0x0DDE0 + i]);
    }
    println!();

    if before == after_1 {
        println!("\nNo change after frame 0");
    } else {
        println!("\nIWRAM changed after frame 0!");
    }
}
