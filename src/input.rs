//! GBA Input/Keypad Handling
//!
//! Handles keypad input including:
//! - D-pad (Up, Down, Left, Right)
//! - Action buttons (A, B)
//! - Shoulder buttons (L, R)
//! - Start, Select

use bitflags::bitflags;

bitflags! {
    /// Keypad state bits
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KeyState: u16 {
        const A = 1 << 0;
        const B = 1 << 1;
        const SELECT = 1 << 2;
        const START = 1 << 3;
        const RIGHT = 1 << 4;
        const LEFT = 1 << 5;
        const UP = 1 << 6;
        const DOWN = 1 << 7;
        const R = 1 << 8;
        const L = 1 << 9;
    }
}

/// GBA Input Handler
pub struct Input {
    keys: KeyState,
    keys_changed: KeyState,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys: KeyState::all(), // All keys "released" (active low)
            keys_changed: KeyState::empty(),
        }
    }

    pub fn reset(&mut self) {
        self.keys = KeyState::all();
        self.keys_changed = KeyState::empty();
    }

    /// Check if a key is pressed
    pub fn is_key_pressed(&self, key: KeyState) -> bool {
        !self.keys.contains(key)
    }

    /// Press a key
    pub fn press_key(&mut self, key: KeyState) {
        if self.keys.contains(key) {
            self.keys -= key;
            self.keys_changed |= key;
        }
    }

    /// Release a key
    pub fn release_key(&mut self, key: KeyState) {
        if !self.keys.contains(key) {
            self.keys |= key;
            self.keys_changed |= key;
        }
    }

    /// Get current key state as register value
    /// GBA key input is active-low: 0 = pressed, 1 = released
    pub fn get_key_register(&self) -> u16 {
        // Bits 0-9: Key states (active low)
        // Bits 10-15: Always set to 1
        (self.keys.bits() & 0x03FF) | 0xFC00
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
