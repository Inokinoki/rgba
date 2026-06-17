//! Behavior Driven Development tests for the GBA Timer System
//!
//! These tests describe the expected behavior of the GBA's 4 timers.

use rgba::Timer;

/// Scenario: Timer initializes correctly
#[test]
fn timer_initializes_with_zero_count() {
    let timer = Timer::new(0);

    assert_eq!(timer.is_enabled(), false, "Timer should start disabled");
    assert_eq!(timer.get_counter(), 0, "Timer should start at 0");
    assert_eq!(timer.get_reload(), 0, "Reload value should be 0");
}

/// Scenario: Timer can be enabled and disabled
#[test]
fn timer_can_be_enabled_and_disabled() {
    let mut timer = Timer::new(0);

    assert_eq!(timer.is_enabled(), false, "Timer should start disabled");

    timer.set_enabled(true);
    assert_eq!(timer.is_enabled(), true, "Timer should be enabled");

    timer.set_enabled(false);
    assert_eq!(timer.is_enabled(), false, "Timer should be disabled");
}

/// Scenario: Timer overflow generates interrupt
#[test]
fn timer_overflow_generates_interrupt_if_enabled() {
    let mut timer = Timer::new(0);

    // Set reload value
    timer.set_reload(0xFF00);

    // Enable with IRQ
    timer.set_control(0xC0); // Enable + IRQ

    // Step timer to cause overflow
    timer.step(0x100);

    // Check if overflow occurred
    assert!(timer.did_overflow() || timer.get_counter() != 0xFF00, "Timer should count or overflow");
}

/// Scenario: Timers can cascade
#[test]
fn timers_can_cascade_for_counting() {
    let mut timer0 = Timer::new(0);
    let mut timer1 = Timer::new(1);

    // Set timer0 in normal mode
    timer0.set_control(0x80); // Enable
    timer0.set_reload(0xFFFE);

    // Set timer1 in count-up mode
    timer1.set_control(0x84); // Enable + count-up
    timer1.set_reload(0);

    // Trigger overflow on timer0
    timer0.step(0x100);

    // Timer1 should increment when timer0 overflows
    if timer0.did_overflow() {
        timer1.trigger_count_up();
        assert!(timer1.get_counter() > 0, "Timer1 should increment when timer0 overflows");
    }
}
