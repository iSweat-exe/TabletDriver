pub mod fallback;
pub mod huion;
pub mod veikk;
pub mod wacom;
pub mod xp_pen;

use crate::drivers::TabletData;

pub trait ReportParser: Send + Sync {
    fn parse(&self, data: &[u8]) -> Option<TabletData>;
}

pub fn create_parser(parser_name: &str) -> Box<dyn ReportParser> {
    if parser_name.contains("XP_PenReportParser") {
        return Box::new(xp_pen::star_g640::XpPenStarG640Parser);
    } else if parser_name.contains("Wacom.Intuos.IntuosReportParser") {
        return Box::new(wacom::intuos::IntuosParser);
    } else if parser_name.contains("Wacom.Intuos.WacomDriverIntuosReportParser") {
        return Box::new(wacom::intuos::WacomDriverIntuosParser);
    } else if parser_name.contains("Wacom.Bamboo.BambooReportParser") {
        return Box::new(wacom::bamboo::BambooParser);
    } else if parser_name.contains("Inspiroy") || parser_name.contains("Giano") {
        return Box::new(huion::inspiroy::InspiroyParser);
    } else if parser_name.contains("Veikk") {
        return Box::new(veikk::generic::VeikkParser);
    } else if parser_name.contains("Gaomon") || parser_name.contains("Tablet") {
        return Box::new(fallback::FallbackParser);
    }

    match parser_name {
        "OpenTabletDriver.Configurations.Parsers.XP_Pen.XP_PenReportParser" => {
            Box::new(xp_pen::star_g640::XpPenStarG640Parser)
        }
        "OpenTabletDriver.Configurations.Parsers.Wacom.Intuos.IntuosReportParser" => {
            Box::new(wacom::intuos::IntuosParser)
        }
        "OpenTabletDriver.Configurations.Parsers.Wacom.Intuos.WacomDriverIntuosReportParser" => {
            Box::new(wacom::intuos::WacomDriverIntuosParser)
        }
        _ => Box::new(fallback::FallbackParser),
    }
}
