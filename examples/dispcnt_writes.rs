use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().dispcnt_write_log_enabled = true;

    for frame in 0..12 {
        gba.run_frame_parallel(&mut fb);
        let log = &gba.mem().dispcnt_write_log;
        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        println!(
            "=== Frame {} DISPCNT=0x{:04X} ({} writes total) ===",
            frame,
            dispcnt,
            log.len()
        );
    }

    let log = &gba.mem().dispcnt_write_log;
    println!("\n=== All DISPCNT writes ===");
    for (i, (pc, byte_off, val)) in log.iter().enumerate() {
        let pc_thumb = pc & !1;
        println!(
            "{:4}: PC=0x{:08X} offset={} val=0x{:02X}",
            i, pc_thumb, byte_off, val
        );
    }
}
