use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut framebuffer = vec![0u32; 240 * 160];

    for frame in 0..195u32 {
        gba.run_frame_parallel(&mut framebuffer);
    }

    let vram = gba.mem().vram();

    let map_base = 0xF800;
    println!("=== Full BG3 map (32x32) at 0xF800 ===");
    for y in 0..32u32 {
        for x in 0..32u32 {
            let i = y * 32 + x;
            let off = map_base + i as usize * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            if tile != 1023 {
                print!("[{:3}/{:?}]", tile, pal);
            } else {
                print!("   .   ");
            }
        }
        println!();
    }

    println!("\n=== Tiles 818-825 data check ===");
    for tile in 818..=825u32 {
        let base = tile as usize * 32;
        let mut nonzero = 0;
        for b in 0..32 {
            if vram[base + b] != 0 {
                nonzero += 1;
            }
        }
        print!("Tile {}: {} nonzero bytes", tile, nonzero);
        if nonzero > 0 {
            print!(" data:");
            for b in 0..32 {
                print!("{:02X}", vram[base + b]);
            }
        }
        println!();
    }

    println!("\n=== Check screen blocks 0-1 for map data ===");
    for sb in 0..2u32 {
        let sb_base = sb * 0x800;
        let mut nonzero = 0;
        for i in 0..1024u32 {
            let off = sb_base as usize + i as usize * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            if entry != 0 {
                nonzero += 1;
            }
        }
        println!(
            "Screen block {} (0x{:04X}): {} nonzero entries",
            sb, sb_base, nonzero
        );
    }

    println!("\n=== Screen block 0 map (first 10x10) ===");
    for y in 0..10u32 {
        for x in 0..10u32 {
            let i = y * 32 + x;
            let entry = u16::from_le_bytes([vram[i as usize * 2], vram[i as usize * 2 + 1]]);
            let tile = entry & 0x3FF;
            print!("{:4}", tile);
        }
        println!();
    }

    println!("\n=== Screen block 1 map (first 10x10) ===");
    let sb1 = 0x0800;
    for y in 0..10u32 {
        for x in 0..10u32 {
            let i = y * 32 + x;
            let off = sb1 + i as usize * 2;
            let entry = u16::from_le_bytes([vram[off], vram[off + 1]]);
            let tile = entry & 0x3FF;
            print!("{:4}", tile);
        }
        println!();
    }
}
