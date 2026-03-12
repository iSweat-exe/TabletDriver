use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};

pub struct Injector {
    enigo: Enigo,
    last_pressure_down: bool,
}

impl Default for Injector {
    fn default() -> Self {
        Self::new()
    }
}

impl Injector {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(&Settings::default()).unwrap(),
            last_pressure_down: false,
        }
    }

    pub fn move_absolute(&mut self, x: f32, y: f32) {
        let _ = self.enigo.move_mouse(x as i32, y as i32, Coordinate::Abs);
    }

    pub fn move_relative(&mut self, dx: f32, dy: f32) {
        if dx.abs() > 0.01 || dy.abs() > 0.01 {
            let _ = self.enigo.move_mouse(dx as i32, dy as i32, Coordinate::Rel);
        }
    }

    pub fn set_left_button(&mut self, is_down: bool) {
        if is_down && !self.last_pressure_down {
            let _ = self.enigo.button(Button::Left, Direction::Press);
        } else if !is_down && self.last_pressure_down {
            let _ = self.enigo.button(Button::Left, Direction::Release);
        }
        self.last_pressure_down = is_down;
    }
}
