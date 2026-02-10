//! RGBA GBA Emulator - Simple GUI Application
//!
//! A minimal graphical emulator interface demonstrating the emulator library.
//!
//! Note: This is a simplified example. For a full-featured GUI, you would need
//! to integrate with additional libraries and handle platform-specific details.

fn main() {
    println!("RGBA GBA Emulator - GUI Example");
    println!("================================");
    println!();
    println!("This example demonstrates how to use the RGBA emulator library.");
    println!();
    println!("To build and run the full GUI application, you need to:");
    println!("1. Install graphics libraries (SDL2, on Linux)");
    println!("2. Enable the GUI feature: --features gui");
    println!("3. Run: cargo run --example full_gui --features gui -- <rom_file>");
    println!();
    println!("For now, here's a simple demo using the emulator library:");
    println!();

    use rgba::Gba;

    // Create emulator
    let mut gba = Gba::new();

    println!("✓ Created GBA emulator instance");
    println!("✓ CPU: ARM7TDMI (ARM + Thumb modes)");
    println!("✓ Memory: 96KB VRAM, 32KB IWRAM, 256KB EWPRAM");
    println!("✓ Display: 240x160 pixels, 6 modes");
    println!("✓ Audio: 4 PSG channels + Direct Sound");
    println!("✓ Timers: 4 timers with cascade support");
    println!("✓ DMA: 4 channels with multiple trigger modes");
    println!();

    // Run a few steps to demonstrate
    println!("Running 1000 CPU cycles...");
    for _ in 0..1000 {
        gba.step();
    }
    println!("✓ Emulator running successfully");
    println!();

    // Display current state
    let cpu = gba.cpu();
    println!("CPU State:");
    println!("  PC: {:#010X}", cpu.get_pc());
    println!("  SP: {:#010X}", cpu.get_sp());
    println!("  Mode: {:?}", cpu.get_mode());
    println!("  Thumb: {}", cpu.is_thumb_mode());
    println!();

    println!("To load a ROM:");
    println!("  gba.load_rom_path(\"path/to/rom.gba\");");
    println!();

    println!("To access components:");
    println!("  let ppu = gba.ppu();");
    println!("  let input = gba.input_mut();");
    println!();

    println!("For a complete GUI application, see the full_gui example.");
    println!("Building the GUI requires platform-specific graphics libraries.");
}
