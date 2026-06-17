use rgba::Gba;
use std::collections::HashMap;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    gba.mem.swi_log_enabled = true;

    for frame in 0..192u32 {
        let log_start = gba.mem.swi_log.len();

        gba.run_frame_parallel(&mut fb);

        let new_swi: Vec<_> = gba.mem.swi_log[log_start..].to_vec();
        if !new_swi.is_empty() {
            let counts: HashMap<u32, usize> = {
                let mut m = HashMap::new();
                for s in &new_swi {
                    *m.entry(*s).or_insert(0) += 1;
                }
                m
            };
            let mut swis: Vec<_> = counts.iter().collect();
            swis.sort_by_key(|&(k, _)| k);
            println!("Frame {:3}: SWIs={:?}", frame, swis);
        }
    }
}
