use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Trace unique PCs and VBlank counter to see what the game does
    gba.mem_mut().dispcnt_write_log_enabled = true;

    // Run 200 frames, track unique PCs at end of each frame
    let mut _last_unique_pcs: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);

        // Read key game state variables
        let vblank_ctr = gba.mem_mut().read_word(0x03007FF8);
        let state = gba.mem_mut().read_word(0x02000074);
        let pc = gba.cpu().get_pc();

        let io = gba.mem().io();
        let dispcnt = u16::from_le_bytes([io[0], io[1]]);
        let ie = gba.mem().interrupt.ie.bits();
        let if_ = gba.mem().interrupt.if_raw.bits();
        let ime = gba.mem().interrupt.ime;

        if frame < 20 || frame % 20 == 0 || dispcnt != 0x0080 && frame > 3 {
            println!("F{:3}: DISPCNT=0x{:04X} PC=0x{:08X} VBLK_ctr=0x{:08X} state=0x{:08X} IE=0x{:04X} IF=0x{:04X} IME={}", 
                frame, dispcnt, pc, vblank_ctr, state, ie, if_, ime);
        }
    }
}
