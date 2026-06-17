use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for _ in 0..10 {
        gba.run_frame_parallel(&mut fb);
    }

    let mut iwram_at_0 = vec![0u8; 16];
    for i in 0..16 {
        iwram_at_0[i] = gba.mem_mut().read_byte(0x03000000 + i as u32);
    }
    let rom_data = gba.mem().rom();
    let rom_src = &rom_data[0x0DDE0..0x0DDF0];

    print!("IWRAM at 0x03000000: ");
    for b in &iwram_at_0 {
        print!("{:02X}", b);
    }
    println!();
    print!("ROM at 0x080D0DE0:   ");
    for b in rom_src {
        print!("{:02X}", b);
    }
    println!();

    if iwram_at_0.as_slice() == rom_src {
        println!("\nIWRAM code matches ROM source ✓");
    } else {
        println!("\n*** IWRAM code DOES NOT match ROM source! ***");
        for i in 0..16 {
            if iwram_at_0[i] != rom_src[i] {
                println!(
                    "  diff at byte {}: IWRAM={:02X} ROM={:02X}",
                    i, iwram_at_0[i], rom_src[i]
                );
            }
        }
    }

    let vram_nonzero: usize = gba.mem().vram().iter().filter(|&&b| b != 0).count();
    println!("\nVRAM nonzero bytes at frame 10: {}", vram_nonzero);
}
