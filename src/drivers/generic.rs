use super::config::TabletConfiguration;
use super::parsers::{create_parser, ReportParser};
use super::{TabletData, NextTabletDriver};

pub struct GenericNextTabletDriver {
    config: TabletConfiguration,
    vid: u16,
    pid: u16,
    parser: Box<dyn ReportParser>,
}

impl GenericNextTabletDriver {
    pub fn new(config: TabletConfiguration, vid: u16, pid: u16) -> Self {
        let parser_name = config
            .digitizer_identifiers
            .first()
            .map(|d| d.report_parser.as_str())
            .unwrap_or("");

        let parser = create_parser(parser_name);

        Self {
            config,
            vid,
            pid,
            parser,
        }
    }
}

impl NextTabletDriver for GenericNextTabletDriver {
    fn get_name(&self) -> &str {
        &self.config.name
    }

    fn get_specs(&self) -> (f32, f32, f32) {
        (
            self.config.specifications.digitizer.max_x,
            self.config.specifications.digitizer.max_y,
            self.config.specifications.pen.max_pressure as f32,
        )
    }

    fn get_physical_specs(&self) -> (f32, f32) {
        (
            self.config.specifications.digitizer.width,
            self.config.specifications.digitizer.height,
        )
    }

    fn get_vid_pid(&self) -> (u16, u16) {
        (self.vid, self.pid)
    }

    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        self.parser.parse(data)
    }
}
