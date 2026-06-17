use rgba::Gba;

fn main() {
    let mut gba = Gba::new();
    gba.load_rom_path("/home/ubuntu/Builds/gba-tests/save/flash128.gba")
        .unwrap();
    gba.mem_mut().set_save_type(rgba::SaveType::Flash128K);

    for _ in 0..500 {
        gba.run_frame();
    }

    println!("R12={:#010X}", gba.cpu().get_reg(12));
}
