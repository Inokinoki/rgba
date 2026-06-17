use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem_mut().vram_log_enabled = true;
    gba.mem_mut().vram_write_log.clear();

    for frame in 0..200 {
        gba.run_frame_parallel(&mut fb);
    }

    let log = &gba.mem().vram_write_log;

    let mut unique_addrs = std::collections::BTreeSet::new();
    for (addr, _pc, _val) in log {
        let offset = (addr - 0x0600_0000) as usize;
        if offset < 0x4000 {
            unique_addrs.insert(offset);
        }
    }

    println!(
        "Unique VRAM char block 0 byte addresses written: {}",
        unique_addrs.len()
    );
    println!(
        "Range: {:#X} - {:#X}",
        unique_addrs.iter().next().unwrap_or(&0),
        unique_addrs.iter().next_back().unwrap_or(&0)
    );

    let max_addr = *unique_addrs.iter().next_back().unwrap_or(&0);
    if max_addr < 0x1000 {
        println!(
            "\n*** VRAM writes only go up to {:#X} — tiles 0-{} ***",
            max_addr,
            max_addr / 32
        );
        println!("*** Background tiles (394+) are NEVER written because the code doesn't reach them! ***");
    }

    let io = gba.mem().io();
    let dispcnt = u16::from_le_bytes([io[0], io[1]]);
    println!(
        "\nDISPCNT at frame 200: {:#06X} (blank={})",
        dispcnt,
        (dispcnt >> 7) & 1
    );

    let vram = gba.mem().vram();
    let mut nonzero_beyond_113 = 0;
    for tile in 114..512 {
        let off = tile * 32;
        for b in 0..32 {
            if vram[off + b] != 0 {
                nonzero_beyond_113 += 1;
                break;
            }
        }
    }
    println!(
        "Nonzero tiles beyond tile 113 in char block 0: {}",
        nonzero_beyond_113
    );
}
