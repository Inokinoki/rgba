use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_bios_path("/tmp/gba_bios.bin").unwrap();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();

    let mut fb = vec![0u32; 240 * 160];

    // Check OBJ layer at frame 300
    for _ in 0..300 {
        gba.run_frame_parallel(&mut fb);
    }

    let oam = gba.mem.oam();
    let mut obj_count = 0;
    let mut text_obj_count = 0;
    for i in (0..0x400).step_by(8) {
        let attr0 = u16::from_le_bytes([oam[i], oam[i + 1]]);
        let attr1 = u16::from_le_bytes([oam[i + 2], oam[i + 3]]);
        let attr2 = u16::from_le_bytes([oam[i + 4], oam[i + 5]]);

        if attr0 & 0x0300 == 0x0200 {
            continue;
        } // disabled/hidden

        let y = attr0 & 0xFF;
        let x = attr1 & 0x1FF;

        if y < 160 && x < 240 {
            obj_count += 1;
            let shape = (attr0 >> 14) & 3;
            let size = (attr1 >> 14) & 3;
            let tile = attr2 & 0x3FF;
            let pri = (attr2 >> 10) & 3;
            let pal = (attr2 >> 12) & 0xF;
            if obj_count <= 20 {
                println!(
                    "OBJ{:02}: y={} x={} tile={} pri={} pal={} shape={} size={}",
                    i / 8,
                    y,
                    x,
                    tile,
                    pri,
                    pal,
                    shape,
                    size
                );
            }
        }
    }
    println!("Active OBJs (on-screen): {}", obj_count);

    // Check rendering at different frames
    println!("\n=== Frame-by-frame FB sample (row 40, cols 100-110) ===");
    for frame in [200, 250, 300, 350, 400, 500].iter() {
        let mut gba2 = Gba::new();
        gba2.load_bios_path("/tmp/gba_bios.bin").unwrap();
        gba2.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
            .unwrap();
        let mut fb2 = vec![0u32; 240 * 160];
        for _ in 0..*frame {
            gba2.run_frame_parallel(&mut fb2);
        }

        let mut colors = Vec::new();
        for x in 90..120 {
            let c = fb2[40 * 240 + x];
            colors.push(format!("{:06X}", c & 0xFFFFFF));
        }

        // Check if this row has non-background pixels
        let unique: std::collections::HashSet<u32> =
            fb2[40 * 240..41 * 240].iter().copied().collect();
        println!(
            "Frame {}: unique colors in row 40: {}  samples: {}",
            frame,
            unique.len(),
            colors.join(" ")
        );
    }
}
