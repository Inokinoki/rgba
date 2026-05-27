use rgba::Gba;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("gba-tests/ppu/hello.gba");
    let frames: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(60);
    let speed_multiplier: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);

    let mut gba = Gba::new();
    match gba.load_rom_path(rom_path) {
        Ok(_) => println!("Loaded ROM: {}", rom_path),
        Err(e) => {
            eprintln!("Failed to load ROM: {}", e);
            std::process::exit(1);
        }
    }

    // Debug: Check ROM loading
    let word0 = gba.mem_mut().read_word(0x08000000);
    let word1 = gba.mem_mut().read_word(0x08000004);
    println!("ROM at 0x08000000: 0x{:08X}", word0);
    println!("ROM at 0x08000004: 0x{:08X}", word1);
    println!("CPU PC after reset: 0x{:08X}", gba.cpu().get_pc());

    // Check interrupt vector table
    let irq_handler = gba.mem_mut().read_word(0x03007FFC);
    println!("IRQ handler at 0x03007FFC: 0x{:08X}", irq_handler);

    // Run a few steps and check state
    let mut last_pc = gba.cpu().get_pc();
    for i in 0..100 {
        let pc = gba.cpu().get_pc();

        // Show all PC values for first 100 steps
        eprintln!("Step {}: PC=0x{:08X}", i, pc);

        gba.step();
        last_pc = gba.cpu().get_pc();
    }

    println!("After steps:");
    println!("  PC: 0x{:08X}", gba.cpu().get_pc());
    println!("  R12: {}", gba.cpu().get_reg(12));
    println!("  DISPCNT: 0x{:04X}", gba.mem_mut().read_half(0x04000000));
    println!("  VCOUNT: {}", gba.mem_mut().read_half(0x04000006));

    let mut framebuffer = vec![0u32; 240 * 160];
    let start = Instant::now();

    if speed_multiplier > 1 {
        // Frame skipping mode: run N frames, render 1
        for frame in 0..frames {
            gba.run_frames_skip_render(&mut framebuffer, speed_multiplier - 1);

            if frame == 0 {
                let non_zero = framebuffer.iter().filter(|&&p| p != 0).count();
                println!("First frame: {} non-zero pixels", non_zero);
            }
        }
    } else {
        // Normal mode: render every frame
        for frame in 0..frames {
            gba.run_frame_parallel(&mut framebuffer);

            if frame == 0 {
                let non_zero = framebuffer.iter().filter(|&&p| p != 0).count();
                println!("First frame: {} non-zero pixels", non_zero);
            }
        }
    }

    let elapsed = start.elapsed();
    let fps = frames as f64 / elapsed.as_secs_f64();
    let effective_fps = fps * speed_multiplier as f64;

    println!("=== Parallel Rendering Benchmark ===");
    println!("ROM: {}", rom_path);
    println!("Frames: {}", frames);
    println!("Speed: {}x", speed_multiplier);
    println!("Time: {:.2}s", elapsed.as_secs_f64());
    println!("Display FPS: {:.1}", fps);
    println!("Effective FPS: {:.1}", effective_fps);
    println!(
        "Non-zero pixels: {}",
        framebuffer.iter().filter(|&&p| p != 0).count()
    );
}
