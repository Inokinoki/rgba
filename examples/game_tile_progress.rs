use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    for checkpoint in [10, 100, 500, 1000, 2000, 5000] {
        let start_frame = if checkpoint == 10 { 0 } else { checkpoint / 2 };
        for _ in start_frame..checkpoint {
            gba.run_frame();
        }

        let vram = gba.mem().vram();

        let max_tile = {
            let mut m = 0;
            for t in 0..1024 {
                let off = t * 32;
                if off + 32 <= vram.len() && vram[off..off + 32].iter().any(|&b| b != 0) {
                    m = t;
                }
            }
            m
        };

        let tile394_sum: u32 = vram[394 * 32..394 * 32 + 32]
            .iter()
            .map(|&b| b as u32)
            .sum();

        let mut total_tile_data = 0u64;
        for i in 0..0x8000u32 {
            total_tile_data += vram[i as usize] as u64;
        }

        println!(
            "Frame {:5}: max_tile={} tile394_sum={} tile_data_total={}",
            checkpoint, max_tile, tile394_sum, total_tile_data
        );
    }
}
