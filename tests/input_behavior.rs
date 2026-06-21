//! Behavior Driven Development tests for the GBA Input System
//!
//! These tests describe the expected behavior of the GBA's keypad input.

use rgba::{Input, KeyState};

/// Scenario: Input initializes with no keys pressed
#[test]
fn input_initializes_with_all_keys_released() {
    let input = Input::new();

    // All keys should be "released" (register reads as all 1s)
    assert_eq!(input.is_key_pressed(KeyState::A), false);
    assert_eq!(input.is_key_pressed(KeyState::B), false);
    assert_eq!(input.is_key_pressed(KeyState::START), false);
    assert_eq!(input.is_key_pressed(KeyState::SELECT), false);
    assert_eq!(input.is_key_pressed(KeyState::UP), false);
    assert_eq!(input.is_key_pressed(KeyState::DOWN), false);
    assert_eq!(input.is_key_pressed(KeyState::LEFT), false);
    assert_eq!(input.is_key_pressed(KeyState::RIGHT), false);
    assert_eq!(input.is_key_pressed(KeyState::L), false);
    assert_eq!(input.is_key_pressed(KeyState::R), false);
}

/// Scenario: Keys can be pressed and released
#[test]
fn keys_can_be_pressed_and_released() {
    let mut input = Input::new();

    // Given: A key is not pressed
    assert_eq!(input.is_key_pressed(KeyState::A), false);

    // When: The key is pressed
    input.press_key(KeyState::A);

    // Then: The key should be pressed
    assert_eq!(input.is_key_pressed(KeyState::A), true);

    // When: The key is released
    input.release_key(KeyState::A);

    // Then: The key should not be pressed
    assert_eq!(input.is_key_pressed(KeyState::A), false);
}

/// Scenario: Multiple keys can be pressed simultaneously
#[test]
fn multiple_keys_can_be_pressed_at_once() {
    let mut input = Input::new();

    // When: Multiple keys are pressed
    input.press_key(KeyState::A);
    input.press_key(KeyState::B);
    input.press_key(KeyState::START);

    // Then: All should be registered as pressed
    assert_eq!(input.is_key_pressed(KeyState::A), true);
    assert_eq!(input.is_key_pressed(KeyState::B), true);
    assert_eq!(input.is_key_pressed(KeyState::START), true);
    assert_eq!(input.is_key_pressed(KeyState::SELECT), false);
}

/// Scenario: Input register returns correct value
#[test]
fn input_register_returns_correct_key_state() {
    let mut input = Input::new();

    // Given: No keys pressed
    let reg = input.get_key_register();
    assert_eq!(reg & 0x3FF, 0x3FF, "All keys should show as released");

    // When: A key is pressed
    input.press_key(KeyState::A);

    // Then: Register should reflect that (active low)
    let reg = input.get_key_register();
    assert_eq!(reg & 0x1, 0, "A key bit should be 0 when pressed");
}

/// Scenario: Input can be reset
#[test]
fn input_reset_clears_all_key_states() {
    let mut input = Input::new();

    // Given: Some keys are pressed
    input.press_key(KeyState::A);
    input.press_key(KeyState::UP);
    input.press_key(KeyState::L);

    // When: Input is reset
    input.reset();

    // Then: All keys should be released
    assert_eq!(input.is_key_pressed(KeyState::A), false);
    assert_eq!(input.is_key_pressed(KeyState::UP), false);
    assert_eq!(input.is_key_pressed(KeyState::L), false);
}

/// Scenario: D-pad directions work correctly
#[test]
fn dpad_directions_work_correctly() {
    let mut input = Input::new();

    // Test each direction
    for (key, name) in [
        (KeyState::UP, "UP"),
        (KeyState::DOWN, "DOWN"),
        (KeyState::LEFT, "LEFT"),
        (KeyState::RIGHT, "RIGHT"),
    ] {
        input.press_key(key);
        assert_eq!(input.is_key_pressed(key), true, "{} should be pressed", name);
        input.release_key(key);
        assert_eq!(input.is_key_pressed(key), false, "{} should be released", name);
    }
}

/// Scenario: Action buttons work correctly
#[test]
fn action_buttons_work_correctly() {
    let mut input = Input::new();

    // A and B buttons
    input.press_key(KeyState::A);
    assert_eq!(input.is_key_pressed(KeyState::A), true);

    input.press_key(KeyState::B);
    assert_eq!(input.is_key_pressed(KeyState::B), true);

    // Both pressed
    assert!(input.is_key_pressed(KeyState::A) && input.is_key_pressed(KeyState::B));
}

/// Scenario: Shoulder buttons work correctly
#[test]
fn shoulder_buttons_work_correctly() {
    let mut input = Input::new();

    input.press_key(KeyState::L);
    input.press_key(KeyState::R);

    assert_eq!(input.is_key_pressed(KeyState::L), true);
    assert_eq!(input.is_key_pressed(KeyState::R), true);
}
