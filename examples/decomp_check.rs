use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        let prev_pc_log = gba.mem.iwram_write_log.len();

        gba.run_frame_parallel(&mut fb);

        // Check if decompression code area was ever executed
        // by checking if any instruction in 0x080D0900-0x080D0C20 was fetched
        // We can't directly check this, but we can check if EWRAM was written
        // by code with PC in that range

        let ewram = gba.mem.wram();
        let v871c =
            u32::from_le_bytes([ewram[0x871C], ewram[0x871D], ewram[0x871E], ewram[0x871F]]);

        if frame <= 5 || frame % 50 == 0 || v871c != 0 {
            // Count total EWRAM non-zero in 0x8000-0xA000
            let mut nz = 0;
            for off in (0x8000..0xA000).step_by(4) {
                let v = u32::from_le_bytes([
                    ewram[off],
                    ewram[off + 1],
                    ewram[off + 2],
                    ewram[off + 3],
                ]);
                if v != 0 {
                    nz += 1;
                }
            }
            println!(
                "Frame {:4}: EWRAM[871C]={:08X} total_nz_8k={}",
                frame, v871c, nz
            );
        }
    }
}
