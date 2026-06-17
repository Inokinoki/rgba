use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    
    let mut fb = vec![0u32; 240 * 160];
    
    // Track key addresses frame by frame to see when they diverge from mGBA
    // mGBA at frame 600: [02000000]=00010280 [02000004]=D1104A02 [02000074]=00000001 [0200007C]=00000000
    let addrs = [0x02000000, 0x02000004, 0x02000040, 0x02000074, 0x0200007C, 0x020000C0, 0x020000F0];
    
    for frame in 0..=601 {
        gba.run_frame_parallel(&mut fb);
        if frame >= 195 && frame <= 205 {
            let vals: Vec<String> = addrs.iter().map(|&a| {
                format!("{:08X}", gba.mem.read_word(a))
            }).collect();
            println!("Frame {:3}: {}", frame, vals.join(" "));
        }
    }
    
    // Just the final frame 600 values
    let vals: Vec<String> = addrs.iter().map(|&a| {
        format!("{:08X}", gba.mem.read_word(a))
    }).collect();
    println!("Frame 600 (final): {}", vals.join(" "));
}
