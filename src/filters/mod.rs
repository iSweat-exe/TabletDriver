pub mod antichatter;

use crate::domain::MappingConfig;

pub trait Filter: Send + Sync {
    fn name(&self) -> &'static str;
    fn process(&mut self, x: f32, y: f32, config: &MappingConfig) -> (f32, f32);
    fn reset(&mut self);
}

pub struct FilterPipeline {
    pub filters: Vec<Box<dyn Filter>>,
}

impl FilterPipeline {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn add(&mut self, filter: Box<dyn Filter>) {
        self.filters.push(filter);
    }

    pub fn process(&mut self, mut x: f32, mut y: f32, config: &MappingConfig) -> (f32, f32) {
        for filter in &mut self.filters {
            let (nx, ny) = filter.process(x, y, config);
            x = nx;
            y = ny;
        }
        (x, y)
    }

    pub fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
        }
    }
}
