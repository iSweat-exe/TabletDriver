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

    // Relative movement accumulators to handle sub-pixel movement
    remainder_x: f32,
    remainder_y: f32,
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
            remainder_x: 0.0,
            remainder_y: 0.0,
        }
    }

    /// Injects an absolute cursor position on the screen.
    /// Used by `Absolute` driver mode.
    ///
    /// # Arguments
    /// * `x` - Target X coordinate in OS pixels.
    /// * `y` - Target Y coordinate in OS pixels.
    pub fn move_absolute(&mut self, target_x: f32, target_y: f32, _u: f32, _v: f32) {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::POINT;
            use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;
            unsafe {
                let mut current_pos = POINT { x: 0, y: 0 };
                if GetCursorPos(&mut current_pos) != 0 {
                    // Calculate the required "leap" to reach the target pixel
                    let dx = target_x - current_pos.x as f32;
                    let dy = target_y - current_pos.y as f32;

                    // Inject via our relative method which handles sub-pixel accumulation
                    self.move_relative(dx, dy);
                }
            }
        }

        #[cfg(not(windows))]
        {
            let _ = self
                .enigo
                .move_mouse(target_x as i32, target_y as i32, Coordinate::Abs);
        }
    }

    pub fn move_relative(&mut self, dx: f32, dy: f32) {
        // Add current drift to new deltas
        let total_dx = dx + self.remainder_x;
        let total_dy = dy + self.remainder_y;

        // Extract integer pixels
        let ix = total_dx.trunc() as i32;
        let iy = total_dy.trunc() as i32;

        // Store leftovers for next frame
        self.remainder_x = total_dx.fract();
        self.remainder_y = total_dy.fract();

        if ix != 0 || iy != 0 {
            let _ = self.enigo.move_mouse(ix, iy, Coordinate::Rel);
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
