use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().dispcnt_write_log_enabled = true;

    for frame in 0..300 {
        gba.run_frame_parallel(&mut fb);
        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let pc = gba.cpu().get_pc();
        let log_len = gba.mem().dispcnt_write_log.len();
        if frame < 20 || frame % 50 == 0 || log_len > gba.mem().dispcnt_write_log.len() - 5 {
            println!(
                "Frame {:3}: DISPCNT=0x{:04X} PC=0x{:08X} (writes={})",
                frame, dispcnt, pc, log_len
            );
        }
    }

    println!(
        "\nTotal DISPCNT writes: {}",
        gba.mem().dispcnt_write_log.len()
    );
    let log = &gba.mem().dispcnt_write_log;
    for (i, (pc, byte_off, val)) in log.iter().enumerate() {
        let pc_thumb = pc & !1;
        if i < 50 || i > log.len() - 10 {
            println!(
                "{:4}: PC=0x{:08X} offset={} val=0x{:02X}",
                i, pc_thumb, byte_off, val
            );
        } else if i == 50 {
            println!("... ({} more) ...", log.len() - 60);
        }
    }
}
