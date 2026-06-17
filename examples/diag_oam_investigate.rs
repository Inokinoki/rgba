use rgba::Gba;

fn main() {
    let rom_path = "/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba";
    let rom_data = std::fs::read(rom_path).unwrap();
    eprintln!("ROM: {} bytes", rom_data.len());

    let frames = 200u32;

    // === PATH A: step() via run_frame() ===
    {
        let mut gba = Gba::new();
        gba.load_rom(rom_data.clone());

        for _ in 0..frames {
            gba.run_frame();
        }

        eprintln!(
            "\n=== PATH A: step() via run_frame(), {} frames ===",
            frames
        );
        eprintln!("PC=0x{:08X}", gba.cpu_pc());

        // Check OAM in memory
        let mem_oam = gba.mem().oam();
        let mut nonzero_oam = 0;
        for i in (0..0x400).step_by(2) {
            let v = u16::from_le_bytes([mem_oam[i], mem_oam[i + 1]]);
            if v != 0 {
                nonzero_oam += 1;
            }
        }
        eprintln!("Memory OAM nonzero halfwords: {}/512", nonzero_oam);

        // Check first 16 sprites in memory OAM
        eprintln!("\nFirst 16 sprites from MEMORY OAM:");
        for s in 0..16 {
            let off = s * 8;
            let a0 = u16::from_le_bytes([mem_oam[off], mem_oam[off + 1]]);
            let a1 = u16::from_le_bytes([mem_oam[off + 2], mem_oam[off + 3]]);
            let a2 = u16::from_le_bytes([mem_oam[off + 4], mem_oam[off + 5]]);
            eprintln!(
                "  Sprite {}: attr0=0x{:04X} attr1=0x{:04X} attr2=0x{:04X}",
                s, a0, a1, a2
            );
        }

        // Check sync_ppu_full (what main.rs does)
        gba.sync_ppu_full();

        // Check OAM in PPU after sync_ppu_full
        let ppu = gba.ppu();
        let ppu_oam = ppu.oam();
        let mut nonzero_ppu = 0;
        for i in (0..0x400).step_by(2) {
            let v = u16::from_le_bytes([ppu_oam[i], ppu_oam[i + 1]]);
            if v != 0 {
                nonzero_ppu += 1;
            }
        }
        eprintln!(
            "\nPPU OAM after sync_ppu_full: nonzero halfwords: {}/512",
            nonzero_ppu
        );

        // Check first 16 sprites from PPU
        eprintln!("\nFirst 16 sprites from PPU OAM (after sync_ppu_full):");
        for s in 0..16 {
            let off = s * 8;
            let a0 = u16::from_le_bytes([ppu_oam[off], ppu_oam[off + 1]]);
            let a1 = u16::from_le_bytes([ppu_oam[off + 2], ppu_oam[off + 3]]);
            let a2 = u16::from_le_bytes([ppu_oam[off + 4], ppu_oam[off + 5]]);
            eprintln!(
                "  Sprite {}: attr0=0x{:04X} attr1=0x{:04X} attr2=0x{:04X}",
                s, a0, a1, a2
            );
        }

        // Count visible sprites on screen
        drop(ppu);
        let mut sprite_pixels = 0u32;
        for y in 0..160u16 {
            for x in 0..240u16 {
                let ppu = gba.ppu();
                if let Some(_) = gba.get_sprite_pixel(ppu, x, y) {
                    sprite_pixels += 1;
                }
            }
        }
        eprintln!("\nSprite pixels on screen (PATH A): {}", sprite_pixels);
    }

    // === PATH B: run_frame_parallel() ===
    {
        let mut gba = Gba::new();
        gba.load_rom(rom_data.clone());

        let mut fb = [0u32; 240 * 160];
        for _ in 0..frames {
            gba.run_frame_parallel(&mut fb);
        }

        eprintln!("\n=== PATH B: run_frame_parallel(), {} frames ===", frames);
        eprintln!("PC=0x{:08X}", gba.cpu_pc());

        // Check OAM in memory
        let mem_oam = gba.mem().oam();
        let mut nonzero_oam = 0;
        for i in (0..0x400).step_by(2) {
            let v = u16::from_le_bytes([mem_oam[i], mem_oam[i + 1]]);
            if v != 0 {
                nonzero_oam += 1;
            }
        }
        eprintln!("Memory OAM nonzero halfwords: {}/512", nonzero_oam);

        // Check first 16 sprites in memory OAM
        eprintln!("\nFirst 16 sprites from MEMORY OAM:");
        for s in 0..16 {
            let off = s * 8;
            let a0 = u16::from_le_bytes([mem_oam[off], mem_oam[off + 1]]);
            let a1 = u16::from_le_bytes([mem_oam[off + 2], mem_oam[off + 3]]);
            let a2 = u16::from_le_bytes([mem_oam[off + 4], mem_oam[off + 5]]);
            eprintln!(
                "  Sprite {}: attr0=0x{:04X} attr1=0x{:04X} attr2=0x{:04X}",
                s, a0, a1, a2
            );
        }

        // Count visible sprites on screen
        let mut sprite_pixels = 0u32;
        for y in 0..160u16 {
            for x in 0..240u16 {
                let ppu = gba.ppu();
                if let Some(_) = gba.get_sprite_pixel(ppu, x, y) {
                    sprite_pixels += 1;
                }
            }
        }
        eprintln!("\nSprite pixels on screen (PATH B): {}", sprite_pixels);
    }

    // === PATH C: step() via run_frame() + manual OAM sync ===
    {
        let mut gba = Gba::new();
        gba.load_rom(rom_data.clone());

        for _ in 0..frames {
            gba.run_frame();
        }

        eprintln!("\n=== PATH C: step() + manual sync_ppu() (not sync_ppu_full) ===",);

        // Use sync_ppu instead of sync_ppu_full to get OAM
        gba.sync_ppu();

        // Check OAM in PPU after sync_ppu
        let ppu = gba.ppu();
        let ppu_oam = ppu.oam();
        let mut nonzero_ppu = 0;
        for i in (0..0x400).step_by(2) {
            let v = u16::from_le_bytes([ppu_oam[i], ppu_oam[i + 1]]);
            if v != 0 {
                nonzero_ppu += 1;
            }
        }
        eprintln!(
            "PPU OAM after sync_ppu: nonzero halfwords: {}/512",
            nonzero_ppu
        );

        // Count visible sprites on screen
        drop(ppu);
        let mut sprite_pixels = 0u32;
        for y in 0..160u16 {
            for x in 0..240u16 {
                let ppu = gba.ppu();
                if let Some(_) = gba.get_sprite_pixel(ppu, x, y) {
                    sprite_pixels += 1;
                }
            }
        }
        eprintln!("Sprite pixels on screen (PATH C): {}", sprite_pixels);
    }
}
