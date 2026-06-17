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
    println!("Total VRAM writes logged: {}", log.len());

    let mut cb0 = 0;
    let mut cb1 = 0;
    let mut cb2 = 0;
    let mut cb3 = 0;
    let mut obj = 0;
    for (addr, _pc, _val) in log {
        let offset = (addr - 0x0600_0000) as usize;
        if offset < 0x4000 {
            cb0 += 1;
        } else if offset < 0x8000 {
            cb1 += 1;
        } else if offset < 0xC000 {
            cb2 += 1;
        } else if offset < 0x10000 {
            cb3 += 1;
        } else {
            obj += 1;
        }
    }
    println!("Char block 0 (0x0000-0x3FFF): {} writes", cb0);
    println!("Char block 1 (0x4000-0x7FFF): {} writes", cb1);
    println!("Char block 2 (0x8000-0xBFFF): {} writes", cb2);
    println!("Char block 3 (0xC000-0xFFFF): {} writes", cb3);
    println!("OBJ area (0x10000+): {} writes", obj);

    let vram = gba.mem().vram();
    let mut nonzero_cb0 = 0;
    for tile in 0..512 {
        let off = tile * 32;
        let mut has = false;
        for b in 0..32 {
            if vram[off + b] != 0 {
                has = true;
                break;
            }
        }
        if has {
            nonzero_cb0 += 1;
        }
    }
    println!("\nNonzero tiles in char block 0: {}/512", nonzero_cb0);

    let mut nonzero_cb0_not_aa = 0;
    for tile in 0..512 {
        let off = tile * 32;
        let mut has = false;
        for b in 0..32 {
            if vram[off + b] != 0 && vram[off + b] != 0xAA {
                has = true;
                break;
            }
        }
        if has {
            nonzero_cb0_not_aa += 1;
        }
    }
    println!(
        "Nonzero non-0xAA tiles in char block 0: {}/512",
        nonzero_cb0_not_aa
    );
}
