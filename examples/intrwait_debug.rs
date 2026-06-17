use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Check the value at the intrwait flag address after a few frames
    for frame in 0..6 {
        gba.run_frame_parallel(&mut fb);
        let flag_addr = gba.mem().intrwait_flag_addr;
        let active = gba.mem().intrwait_active;
        let flag_val = if flag_addr != 0 && flag_addr >= 0x02000000 && flag_addr < 0x04000000 {
            gba.mem_mut().read_half(flag_addr)
        } else {
            0xFFFF
        };
        let pc = gba.cpu().get_pc();
        let ie = gba.mem().interrupt.ie.bits();
        let if_ = gba.mem().interrupt.if_raw.bits();
        println!("F{}: flag_addr=0x{:08X} flag_val=0x{:04X} active={} PC=0x{:08X} IE=0x{:04X} IF=0x{:04X}", 
            frame, flag_addr, flag_val, active, pc, ie, if_);
    }
}
