use rgba::Gba;
use std::io::Write;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let vram = gba.mem().vram();

    let mut f = std::fs::File::create("/tmp/our_vram_195.bin").unwrap();
    f.write_all(&vram[..0x10000]).unwrap();

    println!("Dumped VRAM (0x10000 bytes) to /tmp/our_vram_195.bin");
    println!("Now run mGBA to dump at same frame for comparison");

    let mut palette = gba.mem().palette().to_vec();
    let mut f2 = std::fs::File::create("/tmp/our_palette_195.bin").unwrap();
    f2.write_all(&palette).unwrap();
    println!("Dumped palette to /tmp/our_palette_195.bin");
}
