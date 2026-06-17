use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.reg_snapshot_enabled = true;
    gba.mem.decomp_writes_enabled = true;

    for frame in 0..300u32 {
        let snap_before = gba.mem.reg_snapshots.len();
        let writes_before = gba.mem.decomp_writes.len();

        gba.run_frame_parallel(&mut fb);

        let new_snaps = gba.mem.reg_snapshots.len() - snap_before;
        let new_writes = gba.mem.decomp_writes.len() - writes_before;

        if new_snaps > 0 || new_writes > 0 {
            println!(
                "Frame {:4}: {} snapshots, {} decomp EWRAM writes",
                frame, new_snaps, new_writes
            );
        }

        if frame == 192 || frame == 193 {
            for (i, regs) in gba.mem.reg_snapshots[snap_before..].iter().enumerate() {
                print!("  Snap {}: ", i);
                for j in 0..16 {
                    print!("r{}={:08X} ", j, regs[j]);
                }
                println!();
            }
            let show = 20.min(new_writes);
            for (addr, pc, val) in gba.mem.decomp_writes[writes_before..writes_before + show].iter()
            {
                println!("  Write {:08X}={:02X} PC={:08X}", addr, val, pc);
            }
            if new_writes > 20 {
                println!("  ... and {} more writes", new_writes - 20);
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total snapshots: {}", gba.mem.reg_snapshots.len());
    println!("Total decomp writes: {}", gba.mem.decomp_writes.len());

    println!("\n=== All register snapshots ===");
    for (i, regs) in gba.mem.reg_snapshots.iter().enumerate() {
        print!("Snap {:2}: ", i);
        for j in 0..16 {
            print!("r{}={:08X} ", j, regs[j]);
        }
        println!();
    }

    if !gba.mem.decomp_writes.is_empty() {
        println!("\n=== Decomp write address range ===");
        let min_addr = gba
            .mem
            .decomp_writes
            .iter()
            .map(|(a, _, _)| *a)
            .min()
            .unwrap();
        let max_addr = gba
            .mem
            .decomp_writes
            .iter()
            .map(|(a, _, _)| *a)
            .max()
            .unwrap();
        println!("Address range: {:08X}-{:08X}", min_addr, max_addr);
        let nz = gba
            .mem
            .decomp_writes
            .iter()
            .filter(|(_, _, v)| *v != 0)
            .count();
        println!("Non-zero writes: {}/{}", nz, gba.mem.decomp_writes.len());
    }
}
