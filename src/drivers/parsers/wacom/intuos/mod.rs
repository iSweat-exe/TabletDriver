use crate::drivers::TabletData;
use crate::drivers::parsers::ReportParser;

pub struct IntuosTabletReport {
    pub x: u16,
    pub y: u16,
    pub pressure: u16,
    pub eraser: bool,
    pub near_proximity: bool,
    pub buttons: u8,
    pub hover_distance: u8,
}

impl IntuosTabletReport {
    pub fn new(report: &[u8]) -> Self {
        Self {
            x: u16::from_le_bytes([report[2], report[3]]),
            y: u16::from_le_bytes([report[4], report[5]]),
            pressure: u16::from_le_bytes([report[6], report[7]]),
            eraser: (report[1] & 0x08) != 0,
            near_proximity: (report[1] & 0x80) != 0,
            buttons: (report[1] >> 1) & 0x03,
            hover_distance: report[8],
        }
    }
}

pub struct IntuosParser;

impl ReportParser for IntuosParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.is_empty() { return None; }
        
        match data[0] {
            0x02 => {
                if (data[1] & 0x40) != 0 {
                    let report = IntuosTabletReport::new(data);
                    
                    let raw = data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                    
                    Some(TabletData {
                        status: if report.pressure > 0 { "Contact".to_string() } else if report.near_proximity { "Hover".to_string() } else { "Out of Range".to_string() },
                        x: report.x,
                        y: report.y,
                        pressure: report.pressure,
                        tilt_x: 0,
                        tilt_y: 0,
                        buttons: report.buttons,
                        eraser: report.eraser,
                        hover_distance: report.hover_distance,
                        raw_data: raw,
                        is_connected: true,
                    })
                } else {
                    None
                }
            },
            _ => None
        }
    }
}

pub struct WacomDriverIntuosParser;

impl ReportParser for WacomDriverIntuosParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 2 { return None; }
        // We reuse the basic IntuosParser logic but offset by 1
        IntuosParser.parse(&data[1..])
    }
}
