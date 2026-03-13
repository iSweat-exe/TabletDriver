pub mod antichatter;
pub mod stats;

use crate::core::config::models::MappingConfig;

pub trait Filter: Send + Sync {
    fn name(&self) -> &'static str;
    fn process(&mut self, x: f32, y: f32, config: &MappingConfig) -> (f32, f32);
    fn update_config(&mut self, _config: &MappingConfig) {}
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

    pub fn update_config(&mut self, config: &MappingConfig) {
        for filter in &mut self.filters {
            filter.update_config(config);
        }
    }

    pub fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
        }
    }
}

impl Default for FilterPipeline {
    fn default() -> Self {
        Self::new()
    }
}
