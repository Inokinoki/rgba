use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..20 {
        gba.run_frame_parallel(&mut framebuffer);

        let io = gba.mem().io();
        let iwram = gba.mem().iwram();
        let dispcnt = io[0] as u16 | ((io[1] as u16) << 8);
        let dispstat = io[4] as u16 | ((io[5] as u16) << 8);
        let ie = gba.mem().interrupt.read_register(0x200);
        let irf = gba.mem().interrupt.read_register(0x002);
        let ime = gba.mem().interrupt.read_register(0x208);

        // VBlank counter at IWRAM[0x7FF8]
        let vblank_ctr = u32::from_le_bytes([
            iwram[0x7FF8], iwram[0x7FF9], iwram[0x7FFA], iwram[0x7FFB]
        ]);

        // IRQ handler pointer at IWRAM[0x7FFC]
        let irq_handler = u32::from_le_bytes([
            iwram[0x7FFC], iwram[0x7FFD], iwram[0x7FFE], iwram[0x7FFF]
        ]);

        // Check DMA3 enable state
        let dma3_ctrl = io[0xDE] as u16 | ((io[0xDF] as u16) << 8);

        // EWRAM IO buffer DISPCNT value
        let wram = gba.mem().wram();
        let buf_off = (0x02008D2C - 0x02000000) as usize;
        let buf_dispcnt = if buf_off + 4 <= wram.len() {
            u16::from_le_bytes([wram[buf_off], wram[buf_off + 1]])
        } else { 0 };

        // Check what's at the game's "Smsh" structure (progress indicator)
        // Also check EWRAM around the IO buffer for changes
        let pc = gba.cpu().get_pc();

        println!(
            "F{:2}: DISPCNT={:#06X} DISPSTAT={:#06X} IE={:#06X} IF={:#06X} IME={} DMA3={:#06X} bufDISP={:#06X} VBlankCtr={} IRQh={:#010X} PC={:#010X}",
            frame, dispcnt, dispstat, ie, irf, ime, dma3_ctrl, buf_dispcnt, vblank_ctr, irq_handler, pc        );
    }
}
