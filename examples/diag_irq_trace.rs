use rgba::Gba;
use std::io::Write;

fn write_bmp(path: &str, pixels: &[(u8, u8, u8)], width: u32, height: u32) -> std::io::Result<()> {
    let row_size = (width * 3 + 3) & !3;
    let pixel_data_size = row_size * height;
    let file_size = 54 + pixel_data_size;
    let mut f = std::fs::File::create(path)?;
    f.write_all(b"BM")?;
    f.write_all(&(file_size as u32).to_le_bytes())?;
    f.write_all(&[0u8; 4])?;
    f.write_all(&54u32.to_le_bytes())?;
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(width as i32).to_le_bytes())?;
    f.write_all(&(height as i32).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&24u16.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&pixel_data_size.to_le_bytes())?;
    f.write_all(&2835u32.to_le_bytes())?;
    f.write_all(&2835u32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    for y in (0..height).rev() {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            let (r, g, b) = pixels[idx];
            f.write_all(&[b, g, r])?;
        }
        for _ in 0..(row_size - width * 3) {
            f.write_all(&[0u8])?;
        }
    }
    Ok(())
}

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();

    let mut gba = Gba::new();
    gba.load_rom(rom_data.clone());
    // Run just a few frames with step-level tracing around interrupts
    let mut frame = 0u32;
    let mut total_steps = 0u64;
    let mut found_irq = false;

    // Run frame by frame but step manually to detect when we enter BIOS
    for _ in 0..10_000_000 {
        let pc_before = gba.cpu_pc();
        let mode_before: u32 = gba.cpu_get_cpsr() & 0x1F;

        gba.step();

        total_steps += 1;
        let pc_after = gba.cpu_pc();
        let mode_after: u32 = gba.cpu_get_cpsr() & 0x1F;

        // Detect entering IRQ mode (mode 0x12 = 18)
        if mode_before != 0x12 && mode_after == 0x12 {
            found_irq = true;
            eprintln!("\n=== IRQ taken at step {} ===", total_steps);
            eprintln!(
                "  PC before: 0x{:08X} (mode {:05b})",
                pc_before, mode_before
            );
            eprintln!("  PC after:  0x{:08X} (mode {:05b})", pc_after, mode_after);
            eprintln!("  CPSR: 0x{:08X}", gba.cpu_get_cpsr());
            eprintln!("  LR(r14): 0x{:08X}", gba.cpu().get_reg(14));
            eprintln!("  SP(r13): 0x{:08X}", gba.cpu().get_reg(13));
            let ie = gba.mem().interrupt.ie.bits();
            let if_ = gba.mem().interrupt.if_raw.bits();
            eprintln!("  IE=0x{:04X} IF=0x{:04X}", ie, if_);

            // Now trace the next ~30 instructions in IRQ mode
            for j in 0..30 {
                let pc_t = gba.cpu_pc();
                let thumb = gba.cpu().is_thumb_mode();
                eprintln!("  IRQ step {}: PC=0x{:08X} thumb={}", j, pc_t, thumb);
                gba.step();

                let mode_check = gba.cpu_get_cpsr() & 0x1F;
                if mode_check != 0x12 {
                    eprintln!(
                        "  -> Left IRQ mode at step {}, now mode={:05b} PC=0x{:08X}",
                        j,
                        mode_check,
                        gba.cpu_pc()
                    );
                    break;
                }
                if gba.cpu_pc() >= 0xE000_0000 {
                    eprintln!("  -> INVALID PC=0x{:08X} at step {}", gba.cpu_pc(), j);
                    break;
                }
            }
            break;
        }

        // Check for VBlank counter (scanline 160+)
        let vcount = gba.ppu().get_vcount();
        if vcount == 0 && total_steps > 100 && total_steps % 300000 == 0 {
            frame += 1;
            if frame >= 5 {
                eprintln!(
                    "Frame {} at step {}, PC=0x{:08X}",
                    frame, total_steps, pc_after
                );
                break;
            }
        }

        // Safety check
        if pc_after >= 0xE000_0000 && mode_before != 0x12 {
            eprintln!(
                "INVALID PC=0x{:08X} at step {} (not in IRQ)",
                pc_after, total_steps
            );
            eprintln!(
                "  mode before: {:05b}, mode after: {:05b}",
                mode_before,
                gba.cpu_get_cpsr() & 0x1F
            );
            break;
        }
    }

    if !found_irq {
        eprintln!("No IRQ taken after {} steps", total_steps);
    }

    // Now run frames normally and render
    eprintln!("\n=== Running frames normally ===");
    let mut gba2 = Gba::new();
    gba2.load_rom(rom_data);
    for i in 0..=500u32 {
        gba2.run_frame();
        if i == 3 || i == 10 || i == 50 || i == 100 || i == 200 || i == 500 {
            let pc = gba2.cpu_pc();
            let ie = gba2.mem().interrupt.ie.bits();
            let dc = gba2.ppu().get_dispcnt();
            eprintln!(
                "Frame {}: PC=0x{:08X} IE=0x{:04X} DC=0x{:04X}",
                i, pc, ie, dc
            );
        }
    }
}
