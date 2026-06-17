use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    let vram = gba.mem().vram().to_vec();
    let mut prev_vram = vram;

    for frame in 0..260u32 {
        gba.run_frame_parallel(&mut framebuffer);

        let cur_vram = gba.mem().vram().to_vec();

        let mut tile_writes = 0;
        let mut map_writes = 0;
        let mut obj_writes = 0;
        let mut new_tile_max = 0usize;

        for i in (0..0x18000).step_by(2) {
            if cur_vram[i] != prev_vram[i] || cur_vram[i + 1] != prev_vram[i + 1] {
                if i < 0xC000 {
                    tile_writes += 1;
                    let tile_idx = i / 32;
                    if tile_idx > new_tile_max {
                        new_tile_max = tile_idx;
                    }
                } else if i < 0x10000 {
                    map_writes += 1;
                } else {
                    obj_writes += 1;
                }
            }
        }

        if tile_writes > 0 || map_writes > 0 || obj_writes > 0 {
            let total_nonzero = cur_vram.iter().filter(|&&b| b != 0).count();
            let pc = gba.cpu().get_instruction_pc();
            println!("Frame {}: tile_writes={} (max_tile={}) map_writes={} obj_writes={} total_nonzero={} PC={:#010X}",
                frame, tile_writes, new_tile_max, map_writes, obj_writes, total_nonzero, pc);
        }

        prev_vram = cur_vram;
    }
}
