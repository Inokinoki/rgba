use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    gba.mem_mut().vram_log_enabled = true;

    let mut framebuffer = vec![0u32; 240 * 160];
    let mut frame = 0u32;

    for _ in 0..240 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    gba.input.press_key(KeyState::START);
    for _ in 0..60 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    gba.input.release_key(KeyState::START);
    for _ in 0..120 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }

    // Advance to get more tiles
    for round in 0..100 {
        gba.input.press_key(KeyState::A);
        for _ in 0..3 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
        gba.input.release_key(KeyState::A);
        for _ in 0..12 { gba.run_frame_parallel(&mut framebuffer); frame += 1; }
    }

    let vram = gba.mem().vram();
    
    // Check tiles in different ranges
    for range_name in &["0-127", "128-255", "256-383", "384-511", "512-767", "768-1023"] {
        let (start, end) = match *range_name {
            "0-127" => (0, 128),
            "128-255" => (128, 256),
            "256-383" => (256, 384),
            "384-511" => (384, 512),
            "512-767" => (512, 768),
            "768-1023" => (768, 1024),
            _ => (0, 0),
        };
        let count: u32 = (start..end).map(|t| {
            let off = t as usize * 32;
            if off + 32 > vram.len() { return false; }
            vram[off..off+32].iter().any(|&b| b != 0)
        }).map(|b| if b { 1 } else { 0 }).sum();
        println!("Tiles {}: {} with data", range_name, count);
    }

    // Check VRAM regions
    println!("\nVRAM region summary:");
    for start in (0..vram.len()).step_by(0x4000) {
        let end = (start + 0x4000).min(vram.len());
        let nonzero: usize = vram[start..end].iter().filter(|&&b| b != 0).count();
        println!("  {:#06X}-{:#06X}: {}/{} bytes non-zero", start, end-1, nonzero, end-start);
    }
    
    // Check what the VRAM log says about writes to different regions
    let log = &gba.mem().vram_write_log;
    let mut tile_writes_by_range = [0u32; 8];
    for &(addr, _pc, _val) in log {
        let offset = (addr & 0x1FFFF) as usize;
        let region = offset / 0x4000;
        if region < 8 {
            tile_writes_by_range[region] += 1;
        }
    }
    println!("\nVRAM write log by region:");
    for (i, &count) in tile_writes_by_range.iter().enumerate() {
        println!("  {:#06X}-{:#06X}: {} writes", i * 0x4000, (i+1) * 0x4000 - 1, count);
    }
}
