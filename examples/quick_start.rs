//! Quick Start Example
//!
//! This example demonstrates the basic usage of the RGBA GBA emulator:
//! - Creating a GBA instance
//! - Loading a ROM
//! - Running the emulator
//! - Checking system state

use rgba::Gba;
use std::fs;

fn main() {
    // Create a new GBA instance
    let mut gba = Gba::new();

    println!("🎮 RGBA GBA Emulator - Quick Start");
    println!("==================================");
    println!();

    // Show initial state
    println!("Initial State:");
    println!("  PC: 0x{:08X}", gba.cpu.get_pc());
    println!("  Mode: {:?}", gba.cpu.get_mode());
    println!("  Display Mode: {}", gba.ppu.get_display_mode());
    println!();

    // Load a ROM file (if provided)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let rom_path = args[1].as_str();
        match fs::read(rom_path) {
            Ok(rom_data) => {
                println!("Loading ROM: {} ({} bytes)", rom_path, rom_data.len());
                gba.load_rom(rom_data);
                println!("  ROM loaded successfully!");
                println!();

                // Run one frame
                println!("Running one frame...");
                gba.run_frame();

                // Show state after execution
                println!("State after frame:");
                println!("  PC: 0x{:08X}", gba.cpu.get_pc());
                println!("  SP: 0x{:08X}", gba.cpu.get_sp());
                println!();

                // Run 60 more frames (approximately 1 second)
                println!("Running 60 more frames (~1 second)...");
                for i in 0..60 {
                    gba.run_frame();
                    if (i + 1) % 10 == 0 {
                        print!(".");
                        use std::io::Write;
                        let _ = std::io::stdout().flush();
                    }
                }
                println!();

                println!("Final PC: 0x{:08X}", gba.cpu.get_pc());
            }
            Err(e) => {
                eprintln!("Failed to load ROM: {}", e);
            }
        }
    } else {
        println!("No ROM file provided.");
        println!("Usage: cargo run --example quick_start <path-to-rom.gba>");
        println!();

        // Demonstrate API without ROM
        println!("Demonstrating API features:");

        // Test CPU register access
        gba.cpu.set_reg(0, 0xDEADBEEF);
        println!("  Set R0 = 0x{:08X}", gba.cpu.get_reg(0));

        // Test memory access
        gba.mem.write_byte(0x02000000, 0xAB);
        gba.mem.write_half(0x02000002, 0x1234);
        gba.mem.write_word(0x02000004, 0x56789ABC);
        println!("  Wrote test values to WRAM");

        // Test PPU
        gba.ppu.set_display_enabled(true);
        gba.ppu.set_display_mode(3);
        gba.ppu.set_pixel_mode3(120, 80, 0x7FFF); // White pixel
        println!("  Set display mode 3 and drew a pixel");

        // Test input
        gba.input.press_key(rgba::KeyState::A);
        println!("  Pressed A button");
        println!("  Key register: 0x{:04X}", gba.input.get_key_register());
    }

    println!();
    println!("✅ Example completed successfully!");
}
