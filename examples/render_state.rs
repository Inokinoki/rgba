use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba")
        .unwrap();
    let mut fb = vec![0u32; 240 * 160];

    for frame in 0..500u32 {
        gba.run_frame_parallel(&mut fb);
    }

    gba.sync_ppu_full();

    let dc = gba.ppu().get_dispcnt();
    let mode = dc & 0x7;
    let obj_en = (dc >> 12) & 1;
    let bgs = (dc >> 8) & 0xF;
    println!(
        "DISPCNT: {:04X} mode={} BGs={:04b} OBJ={}",
        dc, mode, bgs, obj_en
    );

    // Check BG control registers
    for bg in 0..4 {
        let bgcnt = gba.ppu().get_bgcnt(bg);
        let priority = bgcnt & 3;
        let tile_base = (bgcnt >> 2) & 3;
        let map_base = (bgcnt >> 8) & 0x1F;
        let size = (bgcnt >> 14) & 3;
        println!(
            "BG{}: BGcnt={:04X} pri={} tile_base={:X} map_base={:X} size={}",
            bg,
            bgcnt,
            priority,
            tile_base * 0x4000,
            map_base * 0x800,
            size
        );
    }

    // Check first 20 OAM entries
    println!("\nFirst 20 OAM entries:");
    let oam = gba.ppu().oam();
    for i in 0..20 {
        let a0 = u16::from_le_bytes([oam[i * 8], oam[i * 8 + 1]]);
        let a1 = u16::from_le_bytes([oam[i * 8 + 2], oam[i * 8 + 3]]);
        let a2 = u16::from_le_bytes([oam[i * 8 + 4], oam[i * 8 + 5]]);

        let y = a0 & 0xFF;
        let shape = (a0 >> 14) & 3;
        let x = a1 & 0x1FF;
        let size_bits = (a1 >> 14) & 3;
        let tile_num = a2 & 0x3FF;
        let priority = (a2 >> 10) & 3;
        let pal_bank = (a2 >> 12) & 0xF;
        let colors_256 = (a2 >> 13) & 1;

        if y < 160 || (y >= 256 && y < 160 + 256) {
            let vis_y = if y >= 256 { y - 256 } else { y };
            if vis_y < 160 {
                println!(
                    "  OAM[{:2}]: y={:3} x={:3} shape={} size={} tile={:4} pri={} pal={:2} 256c={}",
                    i, vis_y, x, shape, size_bits, tile_num, priority, pal_bank, colors_256
                );
            }
        }
    }

    // Check how many sprites are visible
    let mut visible = 0;
    for i in 0..128 {
        let a0 = u16::from_le_bytes([oam[i * 8], oam[i * 8 + 1]]);
        let y = a0 & 0xFF;
        let vis_y = if y >= 256 { y.wrapping_sub(256) } else { y };
        let affine = (a0 >> 8) & 1;
        let disabled = if affine == 0 { (a0 >> 9) & 1 } else { 0 };
        if vis_y < 160 && disabled == 0 {
            visible += 1;
        }
    }
    println!("\nVisible sprites: {}/128", visible);
}
