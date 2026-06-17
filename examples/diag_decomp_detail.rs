use rgba::Gba;

fn main() {
    // Test with NO IRQ delay to see if more tile data gets written
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Temporarily disable IRQ delay by setting a range that nothing will match
    // Actually, we can't easily disable it without modifying source.
    // Instead, let's trace what happens around frame 3-4 when tiles appear

    // Check if the game code decompresses ALL tiles at once or incrementally
    // Frame 3: 51 tiles, Frame 4: 90 tiles, then stuck at 90
    // This suggests the decompression only produces 90 tiles' worth of data

    // Let's check what source data the decompression reads
    // The decompression calls BL to subroutines (the F7FF FEE8 patterns)
    // 0x080D0B88: BL (F7FF FEE8) → target = 0x080D0B88 + 4 + (0xFEE8 << 1 sign-extended)
    // Actually THUMB BL is a 32-bit instruction... let me compute properly

    // THUMB BL encoding: two 16-bit halfwords
    // First:  11110_oooooooooo (high bits of offset)
    // Second: 11111_oooooooooo (low bits of offset)
    // 0xF7FF = 0b1111_0111_1111_1111 → first half: 11110_1111111111 = offset_high
    // 0xFEE8 = 0b1111_1110_1110_1000 → second half: not BL format

    // Actually THUMB BL:
    // First halfword:  1111_0_oooooooooo  (bits 15:11 = 11110)
    // Second halfword: 1111_1_oooooooooo  (bits 15:11 = 11111) for BL
    //                 or 1111_0_oooooooooo (bits 15:11 = 11110) for BLX

    // 0xF7FF = 0b1111_0111_1111_1111 → bits[15:11] = 11110 → first half of BL
    // 0xFEE8 = 0b1111_1110_1110_1000 → bits[15:11] = 11111 → second half of BL
    // This IS a BL instruction!

    // Let me compute the target:
    // offset_high = sign_extend(0x7FF << 12) = 0x7FF000 sign-extended from 22 bits
    // 0x7FF000 = 0b0111_1111_1111_0000_0000_0000 = 22 bits: 0b11_1111_1111_0000_0000_0000
    // Actually the high is 11 bits: 0x7FF = 0b11111111111
    // Signed: sign bit is bit 10 = 1, so this is negative
    // Value = 0x7FF - 0x800 = -1, shifted left by 12 = -0x1000

    // offset_low = 0x2E8 (bits 10:0 of 0xFEE8)
    // target = PC + 4 + offset_high + offset_low
    //        = 0x080D0B8A + 4 + (-0x1000) + 0x2E8
    //        = 0x080D0B8E - 0x1000 + 0x2E8
    //        = 0x080D0B8E - 0xD18
    //        = 0x080CFE76... that's odd (not aligned)

    // Let me just use the actual ROM to understand the decompression better
    // Instead, let me look at it from a different angle:
    // What if the decompression IS working but the CALLING code only asks it to decompress
    // a limited amount?

    // The game is a Chinese translation of Harvest Moon. The title screen should have
    // backgrounds. Let me check if the BG tiles are supposed to come from a different
    // source (like EWRAM → DMA → VRAM).

    // Let's check if there are MORE decompression calls that we're missing
    // by looking at the VRAM write pattern more carefully

    // Run just frame 3 (where tiles first appear)
    for _ in 0..3 {
        gba.run_frame_parallel(&mut fb);
    }

    // Now enable logging and run frame 4
    gba.mem.vram_log_enabled = true;
    gba.run_frame_parallel(&mut fb);

    let log = &gba.mem.vram_write_log;
    eprintln!("Frame 4 VRAM writes: {} entries", log.len());

    // Find the highest tile written with non-zero data
    let mut max_nonzero_tile = 0u32;
    let mut min_zero_tile = 9999u32;

    for &(addr, pc, val) in log {
        let raw_offset = ((addr - 0x0600_0000) % 0x2_0000) as usize;
        let offset = if raw_offset >= 0x1_8000 {
            raw_offset - 0x8000
        } else {
            raw_offset
        };
        let tile_id = (offset / 32) as u32;
        if val != 0 && tile_id > max_nonzero_tile {
            max_nonzero_tile = tile_id;
        }
        if val == 0 && tile_id < min_zero_tile && tile_id > 0 {
            min_zero_tile = tile_id;
        }
    }
    eprintln!("Highest tile with nonzero write: {}", max_nonzero_tile);
    eprintln!(
        "Lowest tile with only-zero writes: {}",
        if min_zero_tile == 9999 {
            0
        } else {
            min_zero_tile
        }
    );

    // Check tiles around the boundary
    eprintln!("\nTile write summary near boundary:");
    for tid in 85..100u32 {
        let mut nz = 0;
        let mut z = 0;
        for &(addr, _, val) in log {
            let raw_offset = ((addr - 0x0600_0000) % 0x2_0000) as usize;
            let offset = if raw_offset >= 0x1_8000 {
                raw_offset - 0x8000
            } else {
                raw_offset
            };
            if offset >= tid as usize * 32 && offset < (tid + 1) as usize * 32 {
                if val != 0 {
                    nz += 1;
                } else {
                    z += 1;
                }
            }
        }
        if nz > 0 || z > 0 {
            eprintln!("  Tile {}: {} nonzero, {} zero writes", tid, nz, z);
        }
    }

    // Check if tiles 344+ are being referenced but no data written
    // Look at screen entries at 0xC000 (BG0)
    gba.sync_ppu_full();
    let vram = gba.mem.vram();
    eprintln!("\nBG0 screen entries (base=0xC000) first 64 entries:");
    for i in 0..64 {
        let addr = 0xC000 + i * 2;
        let entry = u16::from_le_bytes([vram[addr], vram[addr + 1]]);
        let tile = entry & 0x3FF;
        let hflip = (entry >> 10) & 1;
        let vflip = (entry >> 11) & 1;
        let pal = (entry >> 12) & 0xF;
        if i % 32 == 0 {
            eprintln!("\n  Row {}:", i / 32);
        }
        if tile != 0 {
            eprintln!(
                "    [{:2}] tile={:4} hf={} vf={} pal={}",
                i % 32,
                tile,
                hflip,
                vflip,
                pal
            );
        }
    }

    // Now let's understand: maybe the decompression at 0x080D0000 decompresses to EWRAM first,
    // then DMA copies to VRAM? Let's check EWRAM for tile data
    let wram = gba.mem.wram();
    eprintln!("\nEWRAM check for tile-like data at various offsets:");
    for base_off in [0, 0x10000, 0x20000, 0x30000, 0x40000, 0x50000] {
        let mut nonzero = 0;
        for i in 0..0x1000 {
            if base_off + i < wram.len() && wram[base_off + i] != 0 {
                nonzero += 1;
            }
        }
        if nonzero > 0 {
            eprintln!(
                "  0x{:05X}-0x{:05X}: {} nonzero bytes",
                base_off,
                base_off + 0xFFF,
                nonzero
            );
            // Check if it looks like tile data
            let first_nz =
                (0..0x1000).find(|&i| base_off + i < wram.len() && wram[base_off + i] != 0);
            if let Some(fnz) = first_nz {
                eprintln!("    First nonzero at offset 0x{:05X}", base_off + fnz);
            }
        }
    }
}
