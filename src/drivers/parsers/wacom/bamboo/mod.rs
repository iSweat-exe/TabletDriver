use crate::drivers::parsers::ReportParser;
use crate::drivers::TabletData;

pub struct BambooTabletReport {
    pub x: u16,
    pub y: u16,
    pub pressure: u16,
    pub eraser: bool,
    pub near_proximity: bool,
    pub buttons: u8,
    pub aux_buttons: [bool; 4],
}

impl BambooTabletReport {
    pub fn new(report: &[u8]) -> Self {
        let x = u16::from_le_bytes([report[2], report[3]]);
        let y = u16::from_le_bytes([report[4], report[5]]);

        let pressure = if (report[1] & 0x01) != 0 {
            (report[6] as u16) | (((report[7] & 0x03) as u16) << 8)
        } else {
            0
        };

        Self {
            x,
            y,
            pressure,
            eraser: (report[1] & 0x20) != 0,
            near_proximity: (report[1] & 0x80) != 0,
            buttons: (report[1] >> 1) & 0x03,
            aux_buttons: [
                (report[7] & 0x08) != 0,
                (report[7] & 0x10) != 0,
                (report[7] & 0x20) != 0,
                (report[7] & 0x40) != 0,
            ],
        }
    }
}

pub struct BambooParser;

impl ReportParser for BambooParser {
    fn parse(&self, data: &[u8]) -> Option<TabletData> {
        if data.len() < 8 {
            return None;
        }

        match data[0] {
            0x02 => {
                let report = BambooTabletReport::new(data);
                let raw = data
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" ");

                Some(TabletData {
                    status: if report.pressure > 0 {
                        "Contact".to_string()
                    } else if report.near_proximity {
                        "Hover".to_string()
                    } else {
                        "Out of Range".to_string()
                    },
                    x: report.x,
                    y: report.y,
                    pressure: report.pressure,
                    tilt_x: 0,
                    tilt_y: 0,
                    buttons: report.buttons,
                    eraser: report.eraser,
                    hover_distance: 0,
                    raw_data: raw,
                    is_connected: true,
                })
            }
            _ => None,
        }
    }
}
