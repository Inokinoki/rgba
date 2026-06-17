use rgba::Gba;
use rgba::Mode;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/gba矿石镇男孩版 汉化修正版2024.5.5.gba").unwrap();
    let mut fb = vec![0u32; 240 * 160];
    
    for frame in 0..10 {
        gba.run_frame_parallel(&mut fb);
        let pc = gba.cpu.get_pc();
        let mode = gba.cpu.get_mode();
        let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
        println!("Frame {}: PC=0x{:08X} mode={:?} nonzero={}", frame, pc, mode, nonzero);
    }
    
    for frame in (10..=240).step_by(10) {
        for _ in 0..10 { gba.run_frame_parallel(&mut fb); }
        let pc = gba.cpu.get_pc();
        let nonzero: usize = fb.iter().filter(|&&p| p != 0).count();
        println!("Frame {}: PC=0x{:08X} nonzero={}", frame, pc, nonzero);
    }
}
