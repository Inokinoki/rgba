use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    let checkpoints = [1, 10, 50, 100, 150, 200, 250, 300, 400, 500];
    let mut frame = 0u32;
    let mut ci = 0;

    while ci < checkpoints.len() {
        while frame < checkpoints[ci] {
            gba.run_frame_parallel(&mut fb);
            frame += 1;
        }
        let ppu = gba.ppu();
        let pc = gba.cpu_pc();
        println!(
            "frame {}: PC={:08X} DISPCNT={:04X} BG0={:04X} BG1={:04X} BG2={:04X} BG3={:04X}",
            frame,
            pc,
            ppu.get_dispcnt(),
            ppu.get_bgcnt(0),
            ppu.get_bgcnt(1),
            ppu.get_bgcnt(2),
            ppu.get_bgcnt(3)
        );
        ci += 1;
    }
}
