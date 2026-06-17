use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // mGBA frame 0 data (no BIOS):
    // DISPCNT=0x0080 DISPSTAT=0xF00E IE=0x3000 IF=0x0000 IME=1
    // Our frame 0: DISPCNT=0x0000 IE=0x3000 IF=0x0003 IME=1

    // The key difference: mGBA has DISPSTAT=0xF00E (VBlank=1, HBlank=1, VCount=0, VCountIRQ=1)
    // Our DISPSTAT=0x0000

    // Let me check DISPSTAT at the END of frame 0 in more detail
    // VCount at end of frame should be 227 (last visible+blank scanline)
    // DISPSTAT VBlank bit should be set during VBlank period (scanlines 160-226)

    // Run a single scanline at a time and check DISPSTAT
    for scanline in 0..228 {
        gba.run_scanline();
        let io = gba.mem().io();
        let dispstat = u16::from_le_bytes([io[2], io[3]]);
        let vcount = u16::from_le_bytes([io[4], io[5]]);
        let vb = (dispstat >> 0) & 1;
        let hb = (dispstat >> 1) & 1;
        let vm = (dispstat >> 3) & 1;
        if scanline < 5 || scanline > 155 && scanline < 170 || scanline > 225 {
            println!(
                "SL {:3}: DISPSTAT=0x{:04X} VCount={:3} VB={} HB={} VM={}",
                scanline, dispstat, vcount, vb, hb, vm
            );
        }
    }
    println!();

    // Also check PC after the frame
    let pc = gba.cpu().get_pc();
    let ie = gba.mem().interrupt.ie.bits();
    let if_ = gba.mem().interrupt.if_raw.bits();
    let ime = gba.mem().interrupt.ime;
    println!(
        "After 228 SL: PC=0x{:08X} IE=0x{:04X} IF=0x{:04X} IME={}",
        pc, ie, if_, ime
    );
}
