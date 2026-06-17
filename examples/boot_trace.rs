use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    println!("Initial PC: {:08X}", gba.cpu_pc());
    println!("CPSR: {:08X}", gba.cpu_get_cpsr());

    let mut fb = vec![0u32; 240 * 160];
    let mut reached_rom = false;
    for i in 0..2000 {
        gba.run_frame_parallel(&mut fb);
        let pc = gba.cpu_pc();
        if pc >= 0x0800_0000 && !reached_rom {
            println!("*** REACHED ROM at frame {}! PC={:08X}", i + 1, pc);
            reached_rom = true;
        }
        if (i + 1) % 100 == 0 {
            let dispcnt = gba.ppu().get_dispcnt();
            println!("Frame {}: PC={:08X} DISPCNT={:04X}", i + 1, pc, dispcnt);
        }
    }
    if !reached_rom {
        println!("Never reached ROM code in 2000 frames");
    }
}
