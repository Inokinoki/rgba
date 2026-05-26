use rgba::Gba;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("gba-tests/ppu/hello.gba");
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(60);

    let mut gba = Gba::new();
    match gba.load_rom_path(rom_path) {
        Ok(_) => println!("Loaded ROM: {}", rom_path),
        Err(e) => {
            eprintln!("Failed to load ROM: {}", e);
            std::process::exit(1);
        }
    }

    let mut framebuffer = vec![0u32; 240 * 160];
    let start = Instant::now();

    for frame in 0..frames {
        gba.run_frame_parallel(&mut framebuffer);

        if frame == 0 {
            let non_zero = framebuffer.iter().filter(|&&p| p != 0).count();
            println!("First frame: {} non-zero pixels", non_zero);
        }
    }

    let elapsed = start.elapsed();
    let fps = frames as f64 / elapsed.as_secs_f64();

    println!("=== Parallel Rendering Benchmark ===");
    println!("ROM: {}", rom_path);
    println!("Frames: {}", frames);
    println!("Time: {:.2}s", elapsed.as_secs_f64());
    println!("FPS: {:.1}", fps);
    println!(
        "Non-zero pixels: {}",
        framebuffer.iter().filter(|&&p| p != 0).count()
    );
}
