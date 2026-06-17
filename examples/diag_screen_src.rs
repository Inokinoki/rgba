use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // The decompression at 0x080D0xxx writes to VRAM.
    // Let's trace what the function reads (source data) vs what it writes (output).
    //
    // The key code at 0x080D0BFA is: STRB R4, [R1, #0]
    // which stores byte R4 to address R1.
    //
    // For tile 394 (offset 0x3140-0x315F), the writes are all zero.
    // For tile 0 (offset 0x0000-0x001F), the writes are non-zero.
    //
    // This suggests the decompression is WORKING but the INPUT data for
    // tiles 344-472 genuinely decompresses to all zeros.
    //
    // But that can't be right for a game background. Let me check if
    // the SCREEN ENTRIES themselves are the problem.
    //
    // Tile 394 is heavily referenced by BG0. Maybe the screen entries
    // should be different? Let me check the IWRAM for the source data
    // of the screen entries (they were DMA'd from IWRAM).

    for _ in 0..240 {
        gba.run_frame_parallel(&mut fb);
    }
    gba.sync_ppu_full();

    // Check IWRAM for screen entry source data
    let iwram = gba.mem.iwram();
    eprintln!("IWRAM size: {} bytes", iwram.len());

    // The DMA3 transfers from IWRAM to VRAM:
    // 0x03006DD0 -> 0x0600F000 (screen map area)
    // 0x03007E3C -> 0x0600F800
    // Let's check what's at these IWRAM addresses
    eprintln!("\nIWRAM at 0x03006DD0 (DMA source for screen map):");
    let off1 = 0x03006DD0 - 0x03000000;
    for i in (0..64).step_by(2) {
        if off1 + i + 1 < iwram.len() {
            let entry = u16::from_le_bytes([iwram[off1 + i], iwram[off1 + i + 1]]);
            let tile = entry & 0x3FF;
            let pal = (entry >> 12) & 0xF;
            if i % 32 == 0 {
                eprint!("\n  [{:04X}]: ", off1 + i);
            }
            eprint!("{:4}(p{}) ", tile, pal);
        }
    }

    // Actually, let me think about this differently.
    // The screen entries at 0xF000 (BG3) and 0xC000 (BG0) might NOT be
    // from DMA. Let me check the actual VRAM screen entries more carefully.
    //
    // BG0 uses screen_base=0xC000, size=1 (64x32 tiles).
    // This means two screen blocks: block at 0xC000 and block at 0xC800.
    // Total: 64x32 = 2048 entries, 4096 bytes.
    //
    // The DMA to 0x0600F000 copies to the BG3 screen map area (0xF000-0xFFFF).
    // That's 4096 bytes, matching 1024 entries for 32x32.
    //
    // So BG0 screen entries at 0xC000 are from somewhere else.
    // Let me check if there are DMA transfers to 0xC000.

    // Let's check: maybe the decompression at 0x080D0xxx also writes screen entries.
    // Screen entries are at 0xC000-0xCFFF (BG0), 0xD000-0xDFFF (BG1),
    // 0xE000-0xEFFF (BG2), 0xF000-0xFFFF (BG3).
    // All in VRAM range 0xC000-0xFFFF.
    //
    // The VRAM write log showed PC 0x080D0xxx writing to addresses up to 0x06004DBF.
    // That's all in tile data area (0x0000-0xBFFF), not screen entry area (0xC000+).
    //
    // So screen entries are written by DIFFERENT code.

    // Let me check what tile data would look like if we decompressed correctly.
    // For that, let me look at what a real GBA would have at tile 394.
    // Since we can't use mGBA, let me check if the ROM has the raw tile data.

    // The game is Harvest Moon Chinese translation. It likely uses LZ77 compression
    // for tile data. Let me search for LZ77 headers in ROM near the decompression area.

    let rom = gba.mem.rom();
    let mut lz77_headers: Vec<(usize, u32)> = Vec::new();
    for off in (0..rom.len()).step_by(4) {
        if off + 4 <= rom.len() {
            let header = u32::from_le_bytes([rom[off], rom[off + 1], rom[off + 2], rom[off + 3]]);
            if (header & 0xFF) == 0x10 {
                let size = header >> 8;
                if size > 1000 && size < 0x100000 {
                    lz77_headers.push((off, header));
                }
            }
        }
    }

    eprintln!(
        "\n\nLZ77 headers in ROM (size > 1000): {} found",
        lz77_headers.len()
    );
    eprintln!("First 20:");
    for (off, header) in lz77_headers.iter().take(20) {
        let addr = *off as u32 + 0x08000000;
        let size = header >> 8;
        eprintln!("  0x{:08X}: header=0x{:08X} size={}", addr, header, size);
    }

    // Check what the decompression function's source address is.
    // The function at 0x080D0800 reads from a source in ROM.
    // Let me look at its first instructions to understand the calling convention.
    //
    // 0x080D0808: BL 0x080D095C  <- this is the main decompression call
    // The source address should be in R0 or on the stack.

    // Let me check the ROM around 0x080D0900 to see the decompression setup
    eprintln!("\nROM at 0x080D0900-0x080D0960:");
    for off in (0..0x60).step_by(2) {
        let addr = (0x080D0900 - 0x08000000) as usize + off;
        let hw = u16::from_le_bytes([rom[addr], rom[addr + 1]]);
        eprintln!("  0x{:08X}: 0x{:04X}", 0x080D0900 + off as u32, hw);
    }

    // The decompression function is custom code. Let me check the VRAM output.
    // The real question is: are tiles 344-472 SUPPOSED to be empty?
    // If they reference tile 394 which is empty, that means the screen entries
    // reference empty tiles, which means the screen entries themselves might be wrong.
    //
    // Let me check: does the game code at 0x080D0000 set up screen entries too?
    // Or does that happen elsewhere?

    // Let me check what code writes to VRAM 0xC000+ (screen entry area)
    eprintln!("\nLet me check if there's a SWI CpuSet to screen entry area...");
    // From the DMA log: DMA3 from 0x03006DD0 to 0x0600F000
    // That's to BG3's screen map. What about BG0's screen map at 0xC000?

    // Check if the BG0 screen entries look reasonable
    let vram = gba.mem.vram();
    eprintln!("\nBG0 screen entries at 0xC000, first 2 rows:");
    for row in 0..2 {
        for col in 0..64 {
            let entry_addr = 0xC000 + (row * 64 + col) * 2;
            let block = if col >= 32 { 0x800 } else { 0 };
            let adj_col = col % 32;
            let entry_addr = 0xC000 + block + (row * 32 + adj_col) * 2;
            let entry = u16::from_le_bytes([vram[entry_addr], vram[entry_addr + 1]]);
            let tile = entry & 0x3FF;
            if col % 16 == 0 {
                eprint!("\n  row {} col {:2}-: ", row, col);
            }
            eprint!("{:4} ", tile);
        }
        eprintln!();
    }

    // Check if tile 394 has any pattern even though it's zero
    // Maybe it SHOULD be zero and the screen entries are wrong
    eprintln!("\nScreen entries that reference tiles 344-472 in BG0:");
    let mut refs_344_472 = 0;
    for row in 0..32 {
        for col in 0..64 {
            let block = if col >= 32 { 0x800 } else { 0 };
            let adj_col = col % 32;
            let entry_addr = 0xC000 + block + (row * 32 + adj_col) * 2;
            let entry = u16::from_le_bytes([vram[entry_addr], vram[entry_addr + 1]]);
            let tile = entry & 0x3FF;
            if tile >= 344 && tile <= 472 {
                refs_344_472 += 1;
            }
        }
    }
    eprintln!(
        "  {} references to tiles 344-472 out of 2048 entries",
        refs_344_472
    );
}
