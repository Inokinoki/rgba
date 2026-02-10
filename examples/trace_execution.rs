//! Trace ROM execution - see what the CPU is doing
//!
//! This loads a ROM and traces the first few instructions to see if it's running.

use rgba::Gba;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <rom_file>", args[0]);
        println!("\nThis will trace the first 100 CPU instructions");
        println!("to see if the ROM is executing properly.");
        return Ok(());
    }

    let rom_path = &args[1];

    // Create GBA and load ROM
    let mut gba = Gba::new();
    match gba.load_rom_path(rom_path) {
        Ok(_) => println!("✓ Loaded ROM: {}", rom_path),
        Err(e) => {
            eprintln!("✗ Failed to load ROM: {}", e);
            return Err(e);
        }
    }

    println!("\n=== CPU State Before Execution ===");
    let cpu = gba.cpu();
    println!("PC: 0x{:08X}", cpu.get_pc());
    println!("Mode: {}", if cpu.is_thumb_mode() { "Thumb" } else { "ARM" });
    println!("SP: 0x{:08X}", cpu.get_sp());

    // Read first instruction
    let pc = cpu.get_pc();
    let mem = gba.mem();
    let first_word = mem.read_word(pc);
    let first_half = mem.read_half(pc);
    println!("\nFirst instruction at 0x{:08X}:", pc);
    println!("  Word: 0x{:08X}", first_word);
    println!("  Half: 0x{:04X}", first_half);

    println!("\n=== Tracing First 100 Instructions ===");

    for i in 0..100 {
        let pc_before = gba.cpu().get_pc();
        let thumb = gba.cpu().is_thumb_mode();

        // Fetch instruction
        let mem = gba.mem();
        let insn = if thumb {
            mem.read_half(pc_before) as u32
        } else {
            mem.read_word(pc_before)
        };

        // Execute
        gba.step();

        // Check if PC changed
        let pc_after = gba.cpu().get_pc();

        // Print first 20 instructions
        if i < 20 {
            print!("{:3} [{}] PC: 0x{:08X} -> 0x{:08X} | ",
                   i, if thumb { "T" } else { "A" }, pc_before, pc_after);
            if thumb {
                print!("{:04X} ", insn as u16);
            } else {
                print!("{:08X} ", insn);
            }
            println!();
        }

        // Check if we're stuck
        if pc_before == pc_after && i > 0 {
            println!("\n⚠ CPU stalled at 0x{:08X} after {} instructions", pc_after, i + 1);
            println!("This usually means:");
            println!("  1. Unimplemented instruction (infinite loop waiting for interrupt)");
            println!("  2. Branch to self");
            println!("  3. ROM is waiting for something not implemented");
            break;
        }
    }

    println!("\n=== CPU State After 100 Instructions ===");
    let cpu = gba.cpu();
    println!("PC: 0x{:08X}", cpu.get_pc());
    println!("Mode: {}", if cpu.is_thumb_mode() { "Thumb" } else { "ARM" });
    println!("R0-R3: {:08X} {:08X} {:08X} {:08X}",
             cpu.get_reg(0), cpu.get_reg(1), cpu.get_reg(2), cpu.get_reg(3));

    println!("\n=== Display State ===");
    gba.sync_ppu();
    let ppu = gba.ppu();
    println!("Display enabled: {}", ppu.is_display_enabled());
    println!("Display mode: {}", ppu.get_display_mode());
    for bg in 0..4 {
        if ppu.is_bg_enabled(bg) {
            println!("BG{} enabled - BGCNT: 0x{:04X}", bg, ppu.get_bgcnt(bg));
        }
    }

    println!("\n=== IO Register State ===");
    let dispcnt = gba.mem().read_half(0x0400_0000);
    println!("DISPCNT: 0x{:04X}", dispcnt);
    println!("  Mode: {}", dispcnt & 0x7);
    println!("  Display Enable: {}", (dispcnt & 0x80) != 0);
    println!("  BG0: {}", if dispcnt & 0x100 != 0 { "✓" } else { "✗" });
    println!("  BG1: {}", if dispcnt & 0x200 != 0 { "✓" } else { "✗" });
    println!("  BG2: {}", if dispcnt & 0x400 != 0 { "✓" } else { "✗" });
    println!("  BG3: {}", if dispcnt & 0x800 != 0 { "✓" } else { "✗" });
    println!("  OBJ: {}", if dispcnt & 0x1000 != 0 { "✓" } else { "✗" });
    println!("  Win0: {}", if dispcnt & 0x2000 != 0 { "✓" } else { "✗" });
    println!("  Win1: {}", if dispcnt & 0x4000 != 0 { "✓" } else { "✗" });
    println!("  OBJ Win: {}", if dispcnt & 0x8000 != 0 { "✓" } else { "✗" });

    println!("\n=== VRAM & Palette ===");
    let vram = gba.mem().vram();
    let vram_nonzero = vram.iter().filter(|&&b| b != 0).count();
    println!("VRAM non-zero bytes: {}/96KB", vram_nonzero);

    let palette_nonzero = (0..512).filter(|&i| {
        let color = gba.get_palette_color(0, i);
        color != 0
    }).count();
    println!("Palette non-zero entries: {}/512", palette_nonzero);

    Ok(())
}
