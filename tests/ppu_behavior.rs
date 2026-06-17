use rgba::Ppu;

#[test]
fn ppu_initializes_with_correct_defaults() {
    let ppu = Ppu::new();
    assert_eq!(ppu.get_display_mode(), 0);
    assert_eq!(ppu.is_display_enabled(), false);
    assert_eq!(ppu.get_vcount(), 0);
}

#[test]
fn ppu_display_mode_can_be_set() {
    let mut ppu = Ppu::new();
    ppu.set_display_mode(3);
    assert_eq!(ppu.get_display_mode(), 3);
    ppu.set_display_mode(0);
    assert_eq!(ppu.get_display_mode(), 0);
}

#[test]
fn ppu_background_enable_works() {
    let mut ppu = Ppu::new();
    ppu.set_bg_enabled(0, true);
    assert!(ppu.is_bg_enabled(0));
    ppu.set_bg_enabled(0, false);
    assert!(!ppu.is_bg_enabled(0));
}

#[test]
fn ppu_stepping_advances_scanlines() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);

    ppu.step(1232);
    assert_eq!(ppu.get_vcount(), 1);

    for _ in 0..159 {
        ppu.step(1232);
    }
    assert_eq!(ppu.get_vcount(), 160);
    assert!(ppu.is_in_vblank());
}

#[test]
fn ppu_frame_buffer_mode3_accessible() {
    let mut ppu = Ppu::new();
    ppu.set_display_mode(3);
    ppu.set_display_enabled(true);

    for y in 0..160 {
        for x in 0..240 {
            let color = ((x as u16) << 5) | (y as u16);
            ppu.set_pixel_mode3(x, y, color & 0x7FFF);
        }
    }
    let pixel = ppu.get_pixel_mode3(100, 50);
    assert!(pixel != 0);
}

#[test]
fn ppu_reset_clears_state() {
    let mut ppu = Ppu::new();
    ppu.set_display_enabled(true);
    ppu.set_display_mode(4);
    ppu.set_bg_enabled(0, true);
    ppu.reset();
    assert_eq!(ppu.is_display_enabled(), false);
    assert_eq!(ppu.get_display_mode(), 0);
    assert!(!ppu.is_bg_enabled(0));
}

#[test]
fn ppu_mosaic_settings() {
    let mut ppu = Ppu::new();
    ppu.set_bg_mosaic_h(4);
    ppu.set_bg_mosaic_v(4);
    assert_eq!(ppu.get_bg_mosaic_h(), 4);
    assert_eq!(ppu.get_bg_mosaic_v(), 4);
}

#[test]
fn ppu_window1_configuration() {
    let mut ppu = Ppu::new();
    ppu.set_window1_enabled(true);
    ppu.set_window1_left(10);
    ppu.set_window1_right(230);
    ppu.set_window1_top(5);
    ppu.set_window1_bottom(155);
    assert!(ppu.is_window1_enabled());
    assert_eq!(ppu.get_window1_left(), 10);
}

#[test]
fn ppu_blend_configuration() {
    let mut ppu = Ppu::new();
    ppu.set_blending_enabled(true);
    assert!(ppu.is_blending_enabled());
    ppu.set_blend_target_bg(0, true);
    ppu.set_blend_target_bg(1, true);
    assert!(ppu.is_blend_target_bg(0));
    ppu.set_blend_eva(8);
    ppu.set_blend_evb(8);
    assert_eq!(ppu.get_blend_eva(), 8);
    assert_eq!(ppu.get_blend_evb(), 8);
}
