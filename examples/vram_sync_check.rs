use rgba::Gba;
use rgba::KeyState;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut framebuffer = vec![0u32; 240 * 160];
    for _ in 0..240 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().press_key(KeyState::START);
    for _ in 0..60 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    gba.input_mut().release_key(KeyState::START);
    for _ in 0..120 {
        gba.run_frame_parallel(&mut framebuffer);
    }
    for _ in 0..100 {
        gba.input_mut().press_key(KeyState::A);
        for _ in 0..3 {
            gba.run_frame_parallel(&mut framebuffer);
        }
        gba.input_mut().release_key(KeyState::A);
        for _ in 0..12 {
            gba.run_frame_parallel(&mut framebuffer);
        }
    }

    let mem_vram = gba.mem().vram();
    let ppu_vram = gba.ppu().vram();

    let mut diffs = 0u32;
    let mut first_diffs = Vec::new();
    for i in 0..mem_vram.len() {
        if mem_vram[i] != ppu_vram[i] {
            diffs += 1;
            if first_diffs.len() < 20 {
                first_diffs.push((i, mem_vram[i], ppu_vram[i]));
            }
        }
    }
    println!("VRAM diffs between mem and ppu: {}", diffs);
    for (off, m, p) in &first_diffs {
        println!("  {:#06X}: mem={:02X} ppu={:02X}", off, m, p);
    }

    let ppu = gba.ppu();
    println!("\n=== PPU BG state ===");
    for bg in 0..4 {
        println!(
            "  BG{}: bgcnt={:#X} tile_base={:#X} map_base={:#X} hofs={} vofs={}",
            bg,
            ppu.get_bgcnt(bg),
            ppu.get_bg_tile_base(bg),
            ppu.get_bg_map_base(bg),
            ppu.get_bg_hofs(bg),
            ppu.get_bg_vofs(bg)
        );
    }
    println!("  DISPCNT: {:#X}", ppu.get_dispcnt());
    println!("  Mode: {}", ppu.get_display_mode());
}
