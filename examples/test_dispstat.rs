use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    for _ in 0..500000 {
        gba.step();
    }

    let pc = gba.cpu().get_instruction_pc();
    println!("After 500K steps: PC={:#010X}", pc);

    let io = gba.mem.io();
    let dispstat_mem = u16::from_le_bytes([io[4], io[5]]);
    let vcount_mem = u16::from_le_bytes([io[6], io[7]]);
    let ppu_dispstat = gba.ppu().get_dispstat();
    let ppu_vcount = gba.ppu().get_vcount();
    let ppu_vblank = gba.ppu().is_in_vblank();

    println!(
        "Memory DISPSTAT={:#06X} VCOUNT={}",
        dispstat_mem, vcount_mem
    );
    println!(
        "PPU DISPSTAT={:#06X} VCOUNT={} VBlank={}",
        ppu_dispstat, ppu_vcount, ppu_vblank
    );
    println!("PPU hcounter={}", gba.ppu().get_hcounter());

    let mut step = 0u64;
    for _ in 0..10000 {
        gba.step();
        step += 1;

        if step % 1000 == 0 {
            let io = gba.mem.io();
            let ds = u16::from_le_bytes([io[4], io[5]]);
            let vc = u16::from_le_bytes([io[6], io[7]]);
            let pc = gba.cpu().get_instruction_pc();
            let pvb = gba.ppu().is_in_vblank();
            let pvc = gba.ppu().get_vcount();
            println!(
                "  step+{}: PC={:#010X} IO_DS={:#06X} IO_VC={} PPU_VC={} PPU_VB={}",
                step, pc, ds, vc, pvc, pvb
            );
        }
    }
}
