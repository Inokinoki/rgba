use rgba::Gba;
use std::fs;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    // Find "black holes" - pixels that are black (0) but surrounded by non-black
    let mut black_regions: Vec<(u16, u16)> = Vec::new();
    for y in 1..159u16 {
        for x in 1..239u16 {
            let pixel = fb[y as usize * 240 + x as usize];
            if pixel == 0 {
                // Check if surrounded by non-black
                let up = fb[(y - 1) as usize * 240 + x as usize];
                let down = fb[(y + 1) as usize * 240 + x as usize];
                let left = fb[y as usize * 240 + (x - 1) as usize];
                let right = fb[y as usize * 240 + (x + 1) as usize];
                if up != 0 || down != 0 || left != 0 || right != 0 {
                    black_regions.push((x, y));
                }
            }
        }
    }

    println!(
        "Black pixels with non-black neighbors: {}",
        black_regions.len()
    );
    if !black_regions.is_empty() {
        // Show bounding box
        let min_x = black_regions.iter().map(|(x, _)| *x).min().unwrap();
        let max_x = black_regions.iter().map(|(x, _)| *x).max().unwrap();
        let min_y = black_regions.iter().map(|(_, y)| *y).min().unwrap();
        let max_y = black_regions.iter().map(|(_, y)| *y).max().unwrap();
        println!("Bounds: ({},{}) - ({},{})", min_x, min_y, max_x, max_y);

        // Show some sample black pixels
        for (x, y) in black_regions.iter().take(20) {
            println!(
                "  ({},{})=0 neighbors: up={:08X} down={:08X} left={:08X} right={:08X}",
                x,
                y,
                fb[(*y - 1) as usize * 240 + *x as usize],
                fb[(*y + 1) as usize * 240 + *x as usize],
                fb[*y as usize * 240 + (*x - 1) as usize],
                fb[*y as usize * 240 + (*x + 1) as usize]
            );
        }
    }

    // Also check: how many total black pixels in the frame?
    let total_black = fb.iter().filter(|&&p| p == 0).count();
    println!("\nTotal black pixels: {}/{}", total_black, 240 * 160);

    // Check the first few rows to see if there's a pattern
    println!("\nRow 0 pixels (first 20):");
    for x in 0..20 {
        print!("{:08X} ", fb[x]);
    }
    println!();

    // Row 68 (where sprite 13 starts at y=68)
    print!("\nRow 68 (sprite area, x=0..40): ");
    for x in 0..40 {
        print!("{:08X} ", fb[68 * 240 + x]);
    }
    println!();
}
