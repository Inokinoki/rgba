//! Input Demo Example
//!
//! This example demonstrates the Input (keypad) system:
//! - Button press/release
//! - Active-low input handling
//! - Key state register access
//! - All GBA buttons

use rgba::{Gba, KeyState};

fn main() {
    let mut gba = Gba::new();

    println!("🎮 RGBA GBA Emulator - Input Demo");
    println!("=================================");
    println!();

    println!("GBA Button Layout:");
    println!("-----------------");
    println!("        SELECT START");
    println!("          [ ] [ ]");
    println!("");
    println!("      [ ] [ ] [ ] [ ]");
    println!("       L   R                      ←");
    println!("");
    println!("             [ ] [ ]");
    println!("              A   B");
    println!("                 ↓");
    println!();

    // Show initial state
    println!("Initial Key State:");
    println!("-----------------");
    print_key_states(&gba);
    println!();

    // Test individual buttons
    println!("Testing Individual Buttons:");
    println!("---------------------------");

    // Test A button
    println!("\n1. Pressing A button:");
    gba.input.press_key(KeyState::A);
    print!("   Key register: 0x{:04X} (", gba.input.get_key_register());
    println!("{})", key_register_bits(&gba));
    println!("   is_key_pressed(A): {}", gba.input.is_key_pressed(KeyState::A));

    // Test B button
    println!("\n2. Pressing B button:");
    gba.input.press_key(KeyState::B);
    print!("   Key register: 0x{:04X} (", gba.input.get_key_register());
    println!("{})", key_register_bits(&gba));
    println!("   is_key_pressed(B): {}", gba.input.is_key_pressed(KeyState::B));

    // Test Start button
    println!("\n3. Pressing Start button:");
    gba.input.press_key(KeyState::START);
    print!("   Key register: 0x{:04X} (", gba.input.get_key_register());
    println!("{})", key_register_bits(&gba));
    println!("   is_key_pressed(START): {}", gba.input.is_key_pressed(KeyState::START));

    // Test Select button
    println!("\n4. Pressing Select button:");
    gba.input.press_key(KeyState::SELECT);
    print!("   Key register: 0x{:04X} (", gba.input.get_key_register());
    println!("{})", key_register_bits(&gba));

    // Test D-pad
    println!("\n5. Pressing D-pad:");
    gba.input.press_key(KeyState::UP);
    gba.input.press_key(KeyState::RIGHT);
    print!("   Key register: 0x{:04X} (", gba.input.get_key_register());
    println!("{})", key_register_bits(&gba));
    println!("   is_key_pressed(UP): {}", gba.input.is_key_pressed(KeyState::UP));
    println!("   is_key_pressed(RIGHT): {}", gba.input.is_key_pressed(KeyState::RIGHT));

    // Test shoulder buttons
    println!("\n6. Pressing L and R:");
    gba.input.press_key(KeyState::L);
    gba.input.press_key(KeyState::R);
    print!("   Key register: 0x{:04X} (", gba.input.get_key_register());
    println!("{})", key_register_bits(&gba));
    println!("   is_key_pressed(L): {}", gba.input.is_key_pressed(KeyState::L));
    println!("   is_key_pressed(R): {}", gba.input.is_key_pressed(KeyState::R));

    // Release button
    println!("\n7. Releasing A button:");
    gba.input.release_key(KeyState::A);
    print!("   Key register: 0x{:04X} (", gba.input.get_key_register());
    println!("{})", key_register_bits(&gba));
    println!("   is_key_pressed(A): {}", gba.input.is_key_pressed(KeyState::A));

    println!();

    // Test active-low behavior
    println!("Understanding Active-Low Logic:");
    println!("-------------------------------");
    println!("GBA uses active-low input (inverted logic):");
    println!("  0 bit = button is PRESSED");
    println!("  1 bit = button is RELEASED");
    println!();

    // Reset and show clear example
    let mut test_input = rgba::Input::new();

    println!("Example:");
    println!("  Initial state: all keys released");
    println!("    Key register: 0x{:04X} (all 1s = released)", test_input.get_key_register());

    test_input.press_key(KeyState::A);
    println!("  Press A:");
    println!("    Key register: 0x{:04X} (bit 0 = 0 = pressed)", test_input.get_key_register());

    test_input.press_key(KeyState::B);
    println!("  Press B:");
    println!("    Key register: 0x{:04X} (bits 0,1 = 0 = pressed)", test_input.get_key_register());

    test_input.release_key(KeyState::A);
    println!("  Release A:");
    println!("    Key register: 0x{:04X} (bit 0 = 1 = released)", test_input.get_key_register());

    println!();

    // Bit position reference
    println!("Key Register Bit Positions:");
    println!("----------------------------");
    println!("  Bit  0: A button");
    println!("  Bit  1: B button");
    println!("  Bit  2: Select");
    println!("  Bit  3: Start");
    println!("  Bit  4: Right");
    println!("  Bit  5: Left");
    println!("  Bit  6: Up");
    println!("  Bit  7: Down");
    println!("  Bit  8: R button");
    println!("  Bit  9: L button");
    println!("  Bits 10-15: Always 1 (reserved)");
    println!();

    // Simulate a button combo
    println!("Button Combo Example:");
    println!("--------------------");

    let mut combo_input = rgba::Input::new();

    // Simulate the classic A+B+SELECT+START reset combo
    println!("Simulating A+B+Select+Start combo:");
    combo_input.press_key(KeyState::A);
    combo_input.press_key(KeyState::B);
    combo_input.press_key(KeyState::SELECT);
    combo_input.press_key(KeyState::START);

    println!("  Pressed A, B, Select, Start simultaneously");
    print!("  Key register: 0x{:04X} (", combo_input.get_key_register());
    println!("{})", key_register_bits_custom(combo_input.get_key_register()));

    if combo_input.is_key_pressed(KeyState::A) &&
       combo_input.is_key_pressed(KeyState::B) &&
       combo_input.is_key_pressed(KeyState::SELECT) &&
       combo_input.is_key_pressed(KeyState::START) {
        println!("  ✅ All four buttons detected as pressed!");
    }

    println!();

    // Complete input sequence
    println!("Complete Input Sequence Demo:");
    println!("-----------------------------");

    let mut seq_gba = Gba::new();

    println!("Running button press sequence...");

    // Sequence: Up, Up, Down, Down, Left, Right, Left, Right, B, A
    let sequence = vec![
        KeyState::UP, KeyState::UP,
        KeyState::DOWN, KeyState::DOWN,
        KeyState::LEFT, KeyState::RIGHT,
        KeyState::LEFT, KeyState::RIGHT,
        KeyState::B, KeyState::A,
    ];

    for (i, key) in sequence.iter().enumerate() {
        seq_gba.input.press_key(*key);
        println!("  Step {}: Pressed {:?}", i + 1, key);
        // Release previous key
        if i > 0 {
            seq_gba.input.release_key(sequence[i - 1]);
        }
    }

    println!();
    println!("✅ Input demo completed!");
}

fn print_key_states(gba: &Gba) {
    println!("  A:      {}", if gba.input.is_key_pressed(KeyState::A) { "PRESSED" } else { "released" });
    println!("  B:      {}", if gba.input.is_key_pressed(KeyState::B) { "PRESSED" } else { "released" });
    println!("  Select: {}", if gba.input.is_key_pressed(KeyState::SELECT) { "PRESSED" } else { "released" });
    println!("  Start:  {}", if gba.input.is_key_pressed(KeyState::START) { "PRESSED" } else { "released" });
    println!("  Up:     {}", if gba.input.is_key_pressed(KeyState::UP) { "PRESSED" } else { "released" });
    println!("  Down:   {}", if gba.input.is_key_pressed(KeyState::DOWN) { "PRESSED" } else { "released" });
    println!("  Left:   {}", if gba.input.is_key_pressed(KeyState::LEFT) { "PRESSED" } else { "released" });
    println!("  Right:  {}", if gba.input.is_key_pressed(KeyState::RIGHT) { "PRESSED" } else { "released" });
    println!("  L:      {}", if gba.input.is_key_pressed(KeyState::L) { "PRESSED" } else { "released" });
    println!("  R:      {}", if gba.input.is_key_pressed(KeyState::R) { "PRESSED" } else { "released" });
    println!("  Key register: 0x{:04X}", gba.input.get_key_register());
}

fn key_register_bits(gba: &Gba) -> String {
    key_register_bits_custom(gba.input.get_key_register())
}

fn key_register_bits_custom(reg: u16) -> String {
    let bits: Vec<&str> = (0..10).map(|i| {
        if (reg & (1 << i)) == 0 {
            "P" // Pressed
        } else {
            "." // Released
        }
    }).collect();

    format!(
        "A={} B={} Se={} St={} R={} L={} U={} D={} Lb={} Rb={}",
        bits[0], bits[1], bits[2], bits[3],
        bits[4], bits[5], bits[6], bits[7],
        bits[9], bits[8]
    )
}
