use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for _ in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().vram_write_log;

    let mut iwram_writes = 0;
    let mut iwram_pcs = std::collections::BTreeSet::new();
    for (addr, pc, _val) in log {
        if *pc >= 0x0300_0000 && *pc < 0x0400_0000 {
            iwram_writes += 1;
            iwram_pcs.insert(*pc);
        }
    }

    println!("VRAM writes from IWRAM code: {}", iwram_writes);
    println!("Unique IWRAM PCs: {:?}", iwram_pcs);

    if iwram_pcs.is_empty() {
        println!("\n*** No VRAM writes from IWRAM! ***");
        println!("The decompression code at 0x03000000 is NOT writing to VRAM.");
    }

    let vram = gba.mem().vram();
    let tile_394_off = 394 * 32;
    let mut nonzero = 0;
    for b in 0..32 {
        if vram[tile_394_off + b] != 0 {
            nonzero += 1;
        }
    }
    println!(
        "\nTile 394 at VRAM+{:#X}: {} nonzero bytes",
        tile_394_off, nonzero
    );
}
