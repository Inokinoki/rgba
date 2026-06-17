use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/arm/arm.gba")
        .unwrap();

    let mut prev_pc = 0xFFFFFFFFu32;
    let mut same_pc_count = 0u32;
    let mut step = 0u64;

    loop {
        let pc = gba.cpu().get_instruction_pc();
        let dc = gba.ppu().get_dispcnt();
        let vc = gba.ppu().get_vcount();
        let vblank = gba.ppu().is_in_vblank();

        if pc == prev_pc {
            same_pc_count += 1;
            if same_pc_count == 1000 {
                println!(
                    "STUCK at PC={:#010X} after {} steps (DC={:#06X} VC={} VBlank={})",
                    pc, step, dc, vc, vblank
                );
                let r0 = gba.cpu().get_reg(0);
                let r1 = gba.cpu().get_reg(1);
                let r2 = gba.cpu().get_reg(2);
                let r12 = gba.cpu().get_reg(12);
                let lr = gba.cpu().get_reg(14);
                let sp = gba.cpu().get_reg(13);
                println!("  R0={:#010X} R1={:#010X} R2={:#010X}", r0, r1, r2);
                println!("  R12={:#010X} SP={:#010X} LR={:#010X}", r12, sp, lr);
                println!("  Thumb={}", gba.cpu().is_thumb_mode());

                let io = gba.mem.io();
                let dispstat = u16::from_le_bytes([io[4], io[5]]);
                let vcount_io = u16::from_le_bytes([io[6], io[7]]);
                println!("  IO_DISPSTAT={:#06X} IO_VCOUNT={}", dispstat, vcount_io);
                break;
            }
        } else {
            same_pc_count = 0;
            prev_pc = pc;
        }

        gba.step();
        step += 1;

        if step == 100 || step == 1000 || step == 10000 || step == 100000 || step == 500000 {
            println!(
                "Step {}: PC={:#010X} DC={:#06X} VC={} VB={}",
                step, pc, dc, vc, vblank
            );
        }

        if step > 2_000_000 {
            println!("Timeout at step {}", step);
            break;
        }
    }
}
