//! # OS Event Injection
//!
//! This module abstracts the interaction with the operating system's input APIs
//! using the `enigo` crate. It takes normalized screen coordinates and button states
//! from the pipeline and injects them as virtual mouse events.

use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};

/// Handles injecting virtual cursor movements and clicks into the OS.
pub struct Injector {
    enigo: Enigo,
    /// Tracks the previous state of the primary pen button (tip) to avoid spamming
    /// unnecessary "Button Down" events every frame while dragging.
    last_pressure_down: bool,
}

impl Default for Injector {
    fn default() -> Self {
        Self::new()
    }
}

impl Injector {
    /// Instantiates a new Injector using the default OS settings provided by Enigo.
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).unwrap(),
            last_pressure_down: false,
        }
    }

    /// Injects an absolute cursor position on the screen.
    /// Used by `Absolute` driver mode.
    ///
    /// # Arguments
    /// * `x` - Target X coordinate in OS pixels.
    /// * `y` - Target Y coordinate in OS pixels.
    pub fn move_absolute(&mut self, x: f32, y: f32) {
        // <-- Ligne rétablie
        let _ = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs);
    }

    pub fn move_relative(&mut self, dx: f32, dy: f32) {
        // Enigo's relative move functions take integers. To prevent drift from
        // tossing out sub-1.0 pixel remainders, we check if there's *any* notable movement.
        if dx.abs() > 0.01 || dy.abs() > 0.01 {
            let _ = self.enigo.move_mouse(dx as i32, dy as i32, Coordinate::Rel);
        }
    }

    /// Synthesizes a Left Mouse Button click or release event.
    ///
    /// The injector maintains internal state and only fires OS events when the
    /// requested `is_down` state differs from the currently held state, preventing
    /// API spam.
    pub fn set_left_button(&mut self, is_down: bool) {
        if is_down && !self.last_pressure_down {
            let _ = self.enigo.button(Button::Left, Direction::Press);
        } else if !is_down && self.last_pressure_down {
            let _ = self.enigo.button(Button::Left, Direction::Release);
        }
        self.last_pressure_down = is_down;
    }
}
