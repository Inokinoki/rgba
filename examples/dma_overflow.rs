use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    // Run to frame 193 (after the zeroing)
    for _ in 0..193 {
        gba.run_frame_parallel(&mut fb);
    }

    // Check what data DMA3 would write to the overflow area
    // DMA: src=0x0200871C dst=0x05000220 count=128 size=4
    // The overflow portion starts at word 120 (offset 480) from src
    // dst offset 480 = 0x05000220 + 480 = 0x05000400 (mirror of PAL[0])
    // Source for overflow: 0x0200871C + 480 = 0x020088FC

    let src_base = 0x0200871C;
    let overflow_src = src_base + 480; // word 120 out of 128
    println!("DMA3 overflow source: {:08X}", overflow_src);

    // Read EWRAM at the overflow source
    // EWRAM starts at 0x02000000
    let ewram_offset = (overflow_src - 0x02000000) as usize;
    println!("EWRAM offset: {:X}", ewram_offset);

    // Read 32 bytes (8 words) from the overflow source
    for i in 0..8 {
        let addr = overflow_src + i * 4;
        // Use mem.read_word
        let w = gba.mem.read_word(addr);
        println!("  src[120+{}] @ {:08X} = {:08X} -> PAL[{}]", i, addr, w, i);
    }

    // Also check: when does this DMA execute relative to the palette load?
    // The palette load DMA: src=0x084D4AC4 dst=0x05000000 count=128
    // This DMA: src=0x0200871C dst=0x05000220 count=128

    // Check the trigger mode of both DMAs
    let io = gba.mem.io();
    let dma3cnt = u16::from_le_bytes([io[0xDE], io[0xDF]]);
    println!("\nDMA3CNT_H = {:04X}", dma3cnt);
    println!("  enabled: {}", (dma3cnt >> 15) & 1);
    println!("  irq: {}", (dma3cnt >> 14) & 1);
    let timing = (dma3cnt >> 12) & 3;
    println!(
        "  timing: {} ({})",
        timing,
        match timing {
            0 => "Immediate",
            1 => "VBlank",
            2 => "HBlank",
            3 => "Special",
            _ => "?",
        }
    );
    println!("  repeat: {}", (dma3cnt >> 9) & 1);
    println!(
        "  transfer_type: {}",
        if (dma3cnt >> 10) & 1 != 0 {
            "32-bit"
        } else {
            "16-bit"
        }
    );
    println!("  dst_ctrl: {}", (dma3cnt >> 5) & 3);
    println!("  src_ctrl: {}", (dma3cnt >> 7) & 3);

    // Check if the 5th DMA is enabled
    let dma3sad = u32::from_le_bytes([io[0xB4], io[0xB5], io[0xB6], io[0xB7]]);
    let dma3dad = u32::from_le_bytes([io[0xB8], io[0xB9], io[0xBA], io[0xBB]]);
    let dma3cnt_l = u16::from_le_bytes([io[0xBC], io[0xBD]]);
    println!("\nDMA3SAD = {:08X}", dma3sad);
    println!("DMA3DAD = {:08X}", dma3dad);
    println!("DMA3CNT_L = {:04X} (count={})", dma3cnt_l, dma3cnt_l);
}
