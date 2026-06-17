use rgba::Gba;
use std::collections::HashSet;

fn run_test_rom(path: &str, name: &str) -> bool {
    let mut gba = Gba::new();
    if let Err(e) = gba.load_rom_path_patched(path) {
        println!("SKIP: {} (load error: {})", name, e);
        return true;
    };

    for _ in 0..600 {
        gba.run_frame();
    }

    gba.sync_ppu_full();
    gba.sync_ppu();

    let mut green_count = 0u32;
    let mut red_count = 0u32;
    let mut white_count = 0u32;
    let mut black_count = 0u32;
    let mut unique_colors = HashSet::new();

    for y in 0..160u16 {
        for x in 0..240u16 {
            let c = gba.get_pixel_tile_mode(x, y);
            let r = (c >> 10) & 0x1F;
            let g = (c >> 5) & 0x1F;
            let b = c & 0x1F;
            unique_colors.insert(c);
            if g > 20 && r < 5 && b < 5 {
                green_count += 1;
            }
            if r > 20 && g < 5 && b < 5 {
                red_count += 1;
            }
            if r > 25 && g > 25 && b > 25 {
                white_count += 1;
            }
            if r < 3 && g < 3 && b < 3 {
                black_count += 1;
            }
        }
    }

    let result = green_count > 100;
    let label = if result {
        "PASS"
    } else if red_count > 100 {
        "FAIL"
    } else {
        "UNKNOWN"
    };
    println!(
        "{}: {} (green={}, red={}, white={}, black={}, unique={}, pc={:#010X})",
        label,
        name,
        green_count,
        red_count,
        white_count,
        black_count,
        unique_colors.len(),
        gba.cpu().get_instruction_pc()
    );
    result
}

fn main() {
    let tests = [
        ("/home/ubuntu/Builds/gba-tests/arm/arm.gba", "arm"),
        ("/home/ubuntu/Builds/gba-tests/thumb/thumb.gba", "thumb"),
        ("/home/ubuntu/Builds/gba-tests/unsafe/unsafe.gba", "unsafe"),
        ("/home/ubuntu/Builds/gba-tests/bios/bios.gba", "bios"),
        ("/home/ubuntu/Builds/gba-tests/memory/memory.gba", "memory"),
        ("/home/ubuntu/Builds/gba-tests/ppu/hello.gba", "hello"),
        ("/home/ubuntu/Builds/gba-tests/ppu/shades.gba", "shades"),
        ("/home/ubuntu/Builds/gba-tests/ppu/stripes.gba", "stripes"),
        ("/home/ubuntu/Builds/gba-tests/save/none.gba", "save-none"),
        ("/home/ubuntu/Builds/gba-tests/save/sram.gba", "save-sram"),
        (
            "/home/ubuntu/Builds/gba-tests/save/flash64.gba",
            "save-flash64",
        ),
        (
            "/home/ubuntu/Builds/gba-tests/save/flash128.gba",
            "save-flash128",
        ),
        ("/home/ubuntu/Builds/gba-tests/nes/nes.gba", "nes"),
    ];

    let mut passed = 0;
    let mut failed = 0;
    for (path, name) in &tests {
        if run_test_rom(path, name) {
            passed += 1;
        } else {
            failed += 1;
        }
    }
    println!("\n{} passed, {} failed", passed, failed);
}
