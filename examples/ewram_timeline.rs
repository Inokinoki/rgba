use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..1000u32 {
        gba.run_frame_parallel(&mut fb);

        if frame % 100 == 0 || frame == 192 || frame == 191 {
            let ewram = gba.mem.wram();
            let mut nonzero_8000 = 0;
            let mut nonzero_8800 = 0;
            for off in (0x8000..0x8800).step_by(4) {
                let v = u32::from_le_bytes([
                    ewram[off],
                    ewram[off + 1],
                    ewram[off + 2],
                    ewram[off + 3],
                ]);
                if v != 0 {
                    nonzero_8000 += 1;
                }
            }
            for off in (0x8800..0x8A00).step_by(4) {
                let v = u32::from_le_bytes([
                    ewram[off],
                    ewram[off + 1],
                    ewram[off + 2],
                    ewram[off + 3],
                ]);
                if v != 0 {
                    nonzero_8000 += 1;
                    nonzero_8800 += 1;
                }
            }

            // Also check OBJ palette
            let pal = gba.mem.palette();
            let mut obj_nz = 0;
            for i in 0..256 {
                let off = 0x200 + i * 2;
                let c = u16::from_le_bytes([pal[off], pal[off + 1]]);
                if c != 0 {
                    obj_nz += 1;
                }
            }

            println!(
                "Frame {:4}: EWRAM[8000-8800)nz={} EWRAM[8800-8A00)nz={} OBJ_pal_nz={}",
                frame, nonzero_8000, nonzero_8800, obj_nz
            );
        }
    }
}
