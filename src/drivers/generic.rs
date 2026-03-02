use super::{TabletData, TabletDriver};
use super::config::TabletConfiguration;
use super::parsers::{xp_pen, wacom, huion, veikk, fallback};

pub struct GenericTabletDriver {
    config: TabletConfiguration,
}

impl GenericTabletDriver {
    pub fn new(config: TabletConfiguration) -> Self {
        Self { config }
    }
}

impl TabletDriver for GenericTabletDriver {
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

    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        // Find the active parser. In OTD, this is per digitizer, but for now we assume one active generic driver instance covers one device.
        // We look at the first digitizer identifier's parser string to decide the parsing logic.
        // A more robust system would map this string to a trait object or function pointer table.
        
        let parser_name = self.config.digitizer_identifiers.first()
            .map(|d| d.report_parser.as_str())
            .unwrap_or("");
            
        // Check for simplified matches or exact OTD strings
        if parser_name.contains("XP_PenReportParser") {
             return xp_pen::star_g640::parse(data);
        } else if parser_name.contains("Wacom.Intuos.IntuosReportParser") {
             return wacom::intuos::parse_intuos(data);
        } else if parser_name.contains("Wacom.Intuos.WacomDriverIntuosReportParser") {
             return wacom::intuos::parse_wacom_driver_intuos(data);
        } else if parser_name.contains("Wacom.Bamboo.BambooReportParser") {
             return wacom::bamboo::parse_bamboo(data);
        } else if parser_name.contains("Inspiroy") || parser_name.contains("Giano") {
             return huion::inspiroy::parse(data);
        } else if parser_name.contains("Veikk") {
             return veikk::generic::parse(data);
        } else if parser_name.contains("Gaomon") || parser_name.contains("Tablet") {
             // Try fallback for these as they often share the ODM layout
             return fallback::parse(data);
        }

        match parser_name {
            "OpenTabletDriver.Configurations.Parsers.XP_Pen.XP_PenReportParser" => xp_pen::star_g640::parse(data),
            "OpenTabletDriver.Configurations.Parsers.Wacom.Intuos.IntuosReportParser" => wacom::intuos::parse_intuos(data),
            "OpenTabletDriver.Configurations.Parsers.Wacom.Intuos.WacomDriverIntuosReportParser" => wacom::intuos::parse_wacom_driver_intuos(data),
            _ => {
                // If unknown, try fallback anyway but warn?
                // println!("Unknown parser {}. Using fallback.", parser_name);
                fallback::parse(data)
            }
        }
    }
}
